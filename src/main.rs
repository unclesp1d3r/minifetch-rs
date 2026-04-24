// Tests legitimately use .unwrap()/.expect()/panic!-via-assert, so
// exempt them from the strict "no panics in production" lints while
// keeping those lints enforced on the rest of the crate.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::panic, clippy::expect_used))]
// render_bar and the percentage calculations intentionally cast between
// u64/usize/f64 when rendering progress bars. Over the narrow ranges
// involved (0-100 percentages, 0-20 bar cells) none of these casts
// can lose meaningful precision or data. Allow the family crate-wide
// so the render code stays readable.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use console::{Term, measure_text_width, style};
use figlet_rs::FIGlet;
use indicatif::HumanBytes;
use std::collections::HashSet;
use std::path::Path;
use sysinfo::{Components, Disks, Networks, System, Users};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli;

fn main() {
    if let Err(e) = run() {
        // A closed downstream pipe (e.g. `minifetch-rs | head -1`) is
        // normal CLI termination, not an error. Walk the full error
        // chain so a BrokenPipe buried under a future `.context("...")`
        // wrapper still exits 0 instead of printing a scary backtrace.
        let is_broken_pipe = e.chain().any(|src| {
            src.downcast_ref::<std::io::Error>()
                .is_some_and(|io| io.kind() == std::io::ErrorKind::BrokenPipe)
        });
        if is_broken_pipe {
            std::process::exit(0);
        }
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

/// Minimum hostname length (in characters) before figlet rendering kicks
/// in. Below this threshold the hostname is printed as bold styled text,
/// which is both faster (figlet parsing takes a few ms per run) and
/// visually cleaner for short hostnames that figlet would otherwise
/// scatter across a line with lots of empty space.
const FIGLET_MIN_HOSTNAME_LEN: usize = 12;

/// Render a figlet banner for a hostname, falling back to the plain
/// hostname if the embedded font fails to load or the text cannot be
/// rendered. For hostnames shorter than [`FIGLET_MIN_HOSTNAME_LEN`] this
/// short-circuits and returns the hostname unchanged (the print path
/// will style it bold). Never panics.
fn render_banner(hostname: &str) -> String {
    if hostname.chars().count() < FIGLET_MIN_HOSTNAME_LEN {
        return hostname.to_string();
    }
    FIGlet::standard()
        .ok()
        .and_then(|font| font.convert(hostname).map(|fig| fig.to_string()))
        .unwrap_or_else(|| hostname.to_string())
}

#[allow(clippy::too_many_lines)]
fn run() -> Result<()> {
    // Cli exists only so clap handles `--help` / `--version` for us.
    // The parsed value has no fields, so we don't bind it.
    Cli::parse();
    let term = Term::stdout();

    // Only refresh what we actually read. Building a full System::new_all()
    // would also enumerate the process table, per-core CPU stats, and other
    // data that minifetch-rs never displays -- a significant chunk of the
    // startup cost on Linux. Disks, Networks, Components, and Users are
    // constructed as standalone types further down.
    let mut sys = System::new();
    sys.refresh_memory();

    let username = sanitize(&whoami::username().unwrap_or_else(|_| "unknown".to_string()));
    let hostname = sanitize(&System::host_name().unwrap_or_else(|| "N/A".to_string()));

    // Figlet for hostname (graceful fallback handled inside render_banner)
    let banner = render_banner(&hostname);
    term.write_line(&format!("{}", style(banner).cyan()))?;

    let mut content_lines: Vec<InfoLine> = Vec::new();

    // User and Hostname line (first element is always the user@host row,
    // which is rendered centered in the box)
    content_lines.push(InfoLine::plain(
        "User@Host",
        format!("{username}@{hostname}"),
    ));

    // OS and Kernel
    content_lines.push(InfoLine::plain(
        "OS",
        format!(
            "{} {}",
            System::name().unwrap_or_else(|| "N/A".to_string()),
            System::os_version().unwrap_or_else(|| "N/A".to_string())
        ),
    ));
    content_lines.push(InfoLine::plain(
        "Kernel",
        System::kernel_version().unwrap_or_else(|| "N/A".to_string()),
    ));

    // Uptime
    let uptime =
        humantime::format_duration(std::time::Duration::from_secs(System::uptime())).to_string();
    content_lines.push(InfoLine::plain("Uptime", uptime));

    // Logged-in Users
    let users_list = Users::new_with_refreshed_list();
    let users: Vec<String> = users_list.iter().map(|u| u.name().to_string()).collect();
    if !users.is_empty() {
        content_lines.push(InfoLine::plain("Users", users.join(", ")));
    }

    // Load average (replaces the CPU% row, which always read 0 because the
    // two-sample CPU refresh requires a 200ms sleep that would dominate
    // startup latency in a one-shot CLI)
    let la = System::load_average();
    if la.one > 0.0 || la.five > 0.0 || la.fifteen > 0.0 {
        content_lines.push(InfoLine::plain(
            "Load",
            format!("{:.2} {:.2} {:.2}", la.one, la.five, la.fifteen),
        ));
    }

    // Memory (RAM and Swap)
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    if total_memory > 0 {
        // Guard against division by zero: on an extremely exotic
        // container/VM sysinfo could report 0 total RAM. Without the
        // guard the percentage would be NaN/inf and the printed value
        // would be nonsense.
        let memory_percentage = (used_memory as f64 / total_memory as f64) * 100.0;
        content_lines.push(InfoLine::percent(
            "RAM",
            memory_percentage,
            render_bar(memory_percentage as u64, 100, 20),
        ));
    }

    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    if total_swap > 0 {
        let swap_percentage = (used_swap as f64 / total_swap as f64) * 100.0;
        content_lines.push(InfoLine::percent(
            "Swap",
            swap_percentage,
            render_bar(swap_percentage as u64, 100, 20),
        ));
    }

    // Disk Utilization
    let disks = Disks::new_with_refreshed_list();
    let mut seen_disks: HashSet<&Path> = HashSet::new();
    for disk in &disks {
        // `HashSet::insert` returns false if the mount point was already
        // present, so a single call covers both the lookup and the
        // insert without any cloning.
        if !seen_disks.insert(disk.mount_point()) {
            continue;
        }

        let total_space = disk.total_space();
        let available_space = disk.available_space();
        if total_space > 0 && available_space <= total_space {
            let used_space = total_space - available_space;
            let disk_percentage = (used_space as f64 / total_space as f64) * 100.0;
            content_lines.push(InfoLine::percent(
                format!("Disk ({})", sanitize(&disk.name().to_string_lossy())),
                disk_percentage,
                render_bar(disk_percentage as u64, 100, 20),
            ));
        }
    }

    // Network Interfaces and Statistics
    let networks = Networks::new_with_refreshed_list();
    for (interface_name, data) in &networks {
        // Filter for interfaces with activity
        if data.total_received() > 0 || data.total_transmitted() > 0 {
            content_lines.push(InfoLine::plain(
                format!("Net ({})", sanitize(interface_name)),
                format!(
                    "{} (Rx: {}, Tx: {})",
                    data.mac_address(),
                    HumanBytes(data.total_received()),
                    HumanBytes(data.total_transmitted())
                ),
            ));
        }
    }

    // Temperatures
    let components = Components::new_with_refreshed_list();
    let mut temp_count = 0;
    for component in &components {
        if let Some(temp) = component.temperature() {
            // Filter out unrealistic temperatures and limit the number of
            // displayed sensors. The upper bound was widened from 100C to
            // 150C so legitimate hot hardware (Ryzen Tjmax ~95C, NVMe
            // under sustained load, GPUs) is not silently dropped.
            if temp > 0.0 && temp < 150.0 && temp_count < 5 {
                content_lines.push(InfoLine::plain(
                    format!("Temp ({})", sanitize(component.label())),
                    format!("{temp:.1}°C"),
                ));
                temp_count += 1;
            }
        }
    }

    // Compute the box layout once and use it for every row
    let layout = compute_box_layout(&content_lines);
    let box_width = layout.box_width;
    let max_label_width = layout.max_label_width;
    let max_value_width = layout.max_value_width;
    let inner_width = box_width.saturating_sub(2);

    // Print the box top
    term.write_line(&format!(
        "{}{}{}",
        style("┌").black().bright(),
        "─".repeat(inner_width),
        style("┐").black().bright()
    ))?;

    // Print user@hostname line (centered)
    let user_hostname_line_content = &content_lines[0].value;
    let user_hostname_width = measure_text_width(user_hostname_line_content);
    let pad_total = box_width
        .saturating_sub(USER_HOST_OVERHEAD)
        .saturating_sub(user_hostname_width);
    let padding_left = pad_total / 2;
    let padding_right = pad_total - padding_left;
    let centered_user_hostname = format!(
        "{}{}{}",
        " ".repeat(padding_left),
        user_hostname_line_content,
        " ".repeat(padding_right)
    );
    term.write_line(&format!(
        "{} {} {}",
        style("│").black().bright(),
        style(&centered_user_hostname).green(),
        style("│").black().bright()
    ))?;

    term.write_line(&format!(
        "{}{}{}",
        style("├").black().bright(),
        "─".repeat(inner_width),
        style("┤").black().bright()
    ))?;

    // Print other content lines (left-aligned)
    for line in content_lines.iter().skip(1) {
        let label_padding = max_label_width.saturating_sub(measure_text_width(&line.label));
        let value_padding = max_value_width.saturating_sub(line.value_width());

        let value_rendered = line.bar.as_ref().map_or_else(
            || line.value.clone(),
            |bar| format!("{} {}", style(&line.value).white(), style(bar).green()),
        );

        term.write_line(&format!(
            "{} {}{}: {}{}{}",
            style("│").black().bright(),
            style(&line.label).cyan().bright(),
            " ".repeat(label_padding),
            value_rendered,
            " ".repeat(value_padding),
            style("│").black().bright()
        ))?;
    }

    term.write_line(&format!(
        "{}{}{}",
        style("└").black().bright(),
        "─".repeat(inner_width),
        style("┘").black().bright()
    ))?;
    term.write_line(&format!(
        "{}: {}",
        style("Date").green(),
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ))?;

    Ok(())
}

/// Number of fixed characters in a content row (`"│ label: value│"` minus
/// the variable label/value/padding portions).
const BOX_OVERHEAD: usize = 5;

/// Number of fixed characters in the centered user@host row
/// (`"│ centered │"` minus the variable centered portion).
const USER_HOST_OVERHEAD: usize = 4;

/// A single row in the info box. Plain rows carry only `value`; percent
/// rows carry a pre-formatted `value` (e.g. `"45.20%"`) plus a `bar`
/// string produced by [`render_bar`]. Keeping the bar as structured data
/// instead of formatting-then-reparsing avoids the classic anti-pattern
/// where the print loop tries to split a `"45.20% ████"` string back
/// apart on whitespace (which breaks because `render_bar` emits literal
/// trailing spaces for empty cells).
struct InfoLine {
    label: String,
    value: String,
    bar: Option<String>,
}

impl InfoLine {
    /// Plain row with no bar (the most common case).
    fn plain(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            bar: None,
        }
    }

    /// Percent-bar row. `percent` is formatted to two decimals and stored
    /// as `value`; `bar` is stored verbatim and styled at print time.
    fn percent(label: impl Into<String>, percent: f64, bar: String) -> Self {
        Self {
            label: label.into(),
            value: format!("{percent:.2}%"),
            bar: Some(bar),
        }
    }

    /// Visible width of the value column, including the space between
    /// the percent and the bar when a bar is present. Measured in terminal
    /// cells so `measure_text_width` is used rather than byte length.
    fn value_width(&self) -> usize {
        measure_text_width(&self.value) + self.bar.as_ref().map_or(0, |b| 1 + measure_text_width(b))
    }
}

/// Result of [`compute_box_layout`]: total box width plus the widest label
/// and value columns observed across the rows. The print loop pads both
/// columns to those widths so the box renders rectangularly.
#[allow(clippy::struct_field_names)]
struct BoxLayout {
    box_width: usize,
    max_label_width: usize,
    max_value_width: usize,
}

/// Compute the box layout for a slice of [`InfoLine`] rows. The first row
/// is treated as the user@host row and the box is widened so the
/// centered user@host always fits.
fn compute_box_layout(lines: &[InfoLine]) -> BoxLayout {
    let mut max_label = 0usize;
    let mut max_value = 0usize;
    for line in lines {
        max_label = max_label.max(measure_text_width(&line.label));
        max_value = max_value.max(line.value_width());
    }
    let content_width = max_label + max_value + BOX_OVERHEAD;
    let user_hostname_width = lines.first().map_or(0, InfoLine::value_width);
    let box_width = content_width.max(user_hostname_width + USER_HOST_OVERHEAD);
    BoxLayout {
        box_width,
        max_label_width: max_label,
        max_value_width: max_value,
    }
}

/// Strip ASCII/ANSI control characters from OS-sourced strings before
/// printing them. The hostname, username, disk labels, network interface
/// names, and component labels all come from the OS and can contain
/// control bytes (USB volume labels on macOS are the classic source).
/// A hostile or careless label could otherwise inject ANSI escape
/// sequences into minifetch-rs's output.
fn sanitize(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() || *c == ' ').collect()
}

/// Render a horizontal bar of `length` cells filled to `value/max`.
/// Returns an empty string for `length == 0` or `max == 0`. Clamps
/// `value > max` to a fully filled bar so the function never panics.
fn render_bar(value: u64, max: u64, length: usize) -> String {
    if length == 0 || max == 0 {
        return String::new();
    }
    let filled = ((value as f64 / max as f64) * length as f64).round() as usize;
    let filled = filled.min(length);
    let empty = length - filled;
    format!("{}{}", "█".repeat(filled), " ".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_bar_zero_value() {
        assert_eq!(render_bar(0, 100, 10), "          ");
    }

    #[test]
    fn render_bar_full() {
        assert_eq!(render_bar(100, 100, 10), "██████████");
    }

    #[test]
    fn render_bar_half() {
        assert_eq!(render_bar(50, 100, 10), "█████     ");
    }

    #[test]
    fn render_bar_overflow_clamps() {
        // Pre-fix this would underflow on `length - filled_chars`.
        assert_eq!(render_bar(200, 100, 10), "██████████");
    }

    #[test]
    fn render_bar_zero_length() {
        assert_eq!(render_bar(50, 100, 0), "");
    }

    #[test]
    fn render_bar_zero_max() {
        assert_eq!(render_bar(50, 0, 10), "");
    }

    #[test]
    fn info_line_plain_value_width() {
        let line = InfoLine::plain("OS", "Darwin 26.4");
        assert_eq!(line.value_width(), "Darwin 26.4".len());
    }

    #[test]
    fn info_line_percent_value_width_includes_bar_and_space() {
        // "45.20%" = 6 chars, space = 1, "██████" = 6 chars → 13
        let line = InfoLine::percent("RAM", 45.2, "██████".to_string());
        assert_eq!(line.value_width(), 6 + 1 + 6);
    }

    #[test]
    fn box_layout_widens_for_long_user_hostname() {
        // Pre-fix this would underflow on `box_width - user_hostname_width - 2`
        // when the centered user@host row is wider than the rest of the box.
        let lines = vec![
            InfoLine::plain("User@Host", "verylonguser@verylonghostname.example.com"),
            InfoLine::plain("OS", "linux"),
        ];
        let user_host_width = lines[0].value_width();
        let layout = compute_box_layout(&lines);
        assert!(
            layout.box_width >= user_host_width + USER_HOST_OVERHEAD,
            "box_width {} too narrow for user@host width {}",
            layout.box_width,
            user_host_width
        );
    }

    #[test]
    fn box_layout_uses_widest_row_when_content_wins() {
        let lines = vec![
            InfoLine::plain("User@Host", "u@h"),
            InfoLine::plain("LongLabel", "shortvalue"),
        ];
        let layout = compute_box_layout(&lines);
        // max_label = 9 ("LongLabel"), max_value = 10 ("shortvalue")
        assert_eq!(layout.box_width, 9 + 10 + BOX_OVERHEAD);
        assert_eq!(layout.max_value_width, 10);
    }

    #[test]
    fn sanitize_strips_ansi_escapes() {
        assert_eq!(sanitize("\x1b[31mred\x1b[0m"), "[31mred[0m");
    }

    #[test]
    fn sanitize_preserves_spaces_and_printable() {
        assert_eq!(sanitize("Macintosh HD"), "Macintosh HD");
    }

    #[test]
    fn sanitize_strips_newlines_and_tabs() {
        assert_eq!(sanitize("foo\nbar\tbaz"), "foobarbaz");
    }

    #[test]
    fn box_layout_empty_input() {
        let lines: Vec<InfoLine> = vec![];
        let layout = compute_box_layout(&lines);
        // No rows: width is just the BOX_OVERHEAD (zero label + zero value).
        assert_eq!(layout.box_width, BOX_OVERHEAD);
        assert_eq!(layout.max_value_width, 0);
    }
}
