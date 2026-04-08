use anyhow::Result;
use chrono::Local;
use clap::Parser;
use console::{Term, measure_text_width, style};
use figlet_rs::FIGlet;
use indicatif::HumanBytes;
use std::collections::HashSet;
use sysinfo::{Components, Disks, Networks, System, Users};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli;

fn main() {
    if let Err(e) = run() {
        // A closed downstream pipe (e.g. `minifetch-rs | head -1`) is
        // normal CLI termination, not an error. Exit cleanly with status 0.
        if let Some(io_err) = e.downcast_ref::<std::io::Error>()
            && io_err.kind() == std::io::ErrorKind::BrokenPipe
        {
            std::process::exit(0);
        }
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

/// Render a figlet banner for a hostname, falling back to the plain
/// hostname if the embedded font fails to load or the text cannot be
/// rendered. Never panics.
fn render_banner(hostname: &str) -> String {
    FIGlet::standard()
        .ok()
        .and_then(|font| font.convert(hostname).map(|fig| fig.to_string()))
        .unwrap_or_else(|| hostname.to_string())
}

fn run() -> Result<()> {
    let _cli = Cli::parse();
    let term = Term::stdout();

    let mut sys = System::new_all();
    sys.refresh_all();

    let username = whoami::username().unwrap_or_else(|_| "unknown".to_string());
    let hostname = System::host_name().unwrap_or_else(|| "N/A".to_string());

    // Figlet for hostname (graceful fallback handled inside render_banner)
    let banner = render_banner(&hostname);
    term.write_line(&format!("{}", style(banner).cyan()))?;

    let mut content_lines: Vec<(String, String)> = Vec::new(); // (label, value)

    // User and Hostname line (unstyled for width calculation)
    let unstyled_user_hostname = format!("{}@{}", username, hostname);
    content_lines.push(("User@Host".to_string(), unstyled_user_hostname));

    // OS and Kernel
    content_lines.push((
        "OS".to_string(),
        format!(
            "{} {}",
            System::name().unwrap_or_else(|| "N/A".to_string()),
            System::os_version().unwrap_or_else(|| "N/A".to_string())
        ),
    ));
    content_lines.push((
        "Kernel".to_string(),
        System::kernel_version().unwrap_or_else(|| "N/A".to_string()),
    ));

    // Uptime
    let uptime =
        humantime::format_duration(std::time::Duration::from_secs(System::uptime())).to_string();
    content_lines.push(("Uptime".to_string(), uptime));

    // Logged-in Users
    let users_list = Users::new_with_refreshed_list();
    let users: Vec<String> = users_list.iter().map(|u| u.name().to_string()).collect();
    if !users.is_empty() {
        content_lines.push(("Users".to_string(), users.join(", ")));
    }

    // Load average (replaces the CPU% row, which always read 0 because the
    // two-sample CPU refresh requires a 200ms sleep that would dominate
    // startup latency in a one-shot CLI)
    let la = System::load_average();
    if la.one > 0.0 || la.five > 0.0 || la.fifteen > 0.0 {
        content_lines.push((
            "Load".to_string(),
            format!("{:.2} {:.2} {:.2}", la.one, la.five, la.fifteen),
        ));
    }

    // Memory (RAM and Swap)
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_percentage = (used_memory as f64 / total_memory as f64) * 100.0;
    content_lines.push((
        "RAM".to_string(),
        format!(
            "{:.2}% {}",
            memory_percentage,
            render_bar(memory_percentage as u64, 100, 20)
        ),
    ));

    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    if total_swap > 0 {
        let swap_percentage = (used_swap as f64 / total_swap as f64) * 100.0;
        content_lines.push((
            "Swap".to_string(),
            format!(
                "{:.2}% {}",
                swap_percentage,
                render_bar(swap_percentage as u64, 100, 20)
            ),
        ));
    }

    // Disk Utilization
    let disks = Disks::new_with_refreshed_list();
    let mut seen_disks = HashSet::new();
    for disk in disks.iter() {
        let mount_point = disk.mount_point().to_string_lossy().to_string();
        if seen_disks.contains(&mount_point) {
            continue;
        }
        seen_disks.insert(mount_point.clone());

        let total_space = disk.total_space();
        let available_space = disk.available_space();
        if total_space > 0 && available_space <= total_space {
            let used_space = total_space - available_space;
            let disk_percentage = (used_space as f64 / total_space as f64) * 100.0;
            content_lines.push((
                format!("Disk ({})", disk.name().to_string_lossy()),
                format!(
                    "{:.2}% {}",
                    disk_percentage,
                    render_bar(disk_percentage as u64, 100, 20)
                ),
            ));
        }
    }

    // Network Interfaces and Statistics
    let networks = Networks::new_with_refreshed_list();
    for (interface_name, data) in networks.iter() {
        // Filter for interfaces with activity
        if data.total_received() > 0 || data.total_transmitted() > 0 {
            content_lines.push((
                format!("Net ({})", interface_name),
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
    for component in components.iter() {
        if let Some(temp) = component.temperature() {
            // Filter out unrealistic temperatures and limit the number of displayed sensors
            if temp > 0.0 && temp < 100.0 && temp_count < 5 {
                content_lines.push((
                    format!("Temp ({})", component.label()),
                    format!("{:.1}°C", temp),
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
    let user_hostname_line_content = &content_lines[0].1;
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
    for (label, value) in content_lines.iter().skip(1) {
        let styled_value = if label.contains("CPU")
            || label.contains("RAM")
            || label.contains("Swap")
            || label.contains("Disk")
        {
            let parts: Vec<&str> = value.split(' ').collect();
            if parts.len() == 2 {
                format!("{}{}", style(parts[0]).white(), style(parts[1]).green())
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        };
        let current_value_width = measure_text_width(&styled_value);
        let value_padding = max_value_width.saturating_sub(current_value_width);
        let label_padding = max_label_width.saturating_sub(measure_text_width(label));

        term.write_line(&format!(
            "{} {}{}: {}{}{}",
            style("│").black().bright(),
            style(label).cyan().bright(),
            " ".repeat(label_padding),
            styled_value,
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

/// Result of [`compute_box_layout`]: total box width plus the widest label
/// and value columns observed across the rows. The print loop pads both
/// columns to those widths so the box renders rectangularly.
struct BoxLayout {
    box_width: usize,
    max_label_width: usize,
    max_value_width: usize,
}

/// Compute the box layout for a slice of `(label, value)` rows. The first
/// row is treated as the user@host row and the box is widened so the
/// centered user@host always fits.
fn compute_box_layout(lines: &[(String, String)]) -> BoxLayout {
    let mut max_label = 0usize;
    let mut max_value = 0usize;
    for (label, value) in lines {
        max_label = max_label.max(measure_text_width(label));
        max_value = max_value.max(measure_text_width(value));
    }
    let content_width = max_label + max_value + BOX_OVERHEAD;
    let user_hostname_width = lines
        .first()
        .map(|(_, v)| measure_text_width(v))
        .unwrap_or(0);
    let box_width = content_width.max(user_hostname_width + USER_HOST_OVERHEAD);
    BoxLayout {
        box_width,
        max_label_width: max_label,
        max_value_width: max_value,
    }
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
    fn box_layout_widens_for_long_user_hostname() {
        // Pre-fix this would underflow on `box_width - user_hostname_width - 2`
        // when the centered user@host row is wider than the rest of the box.
        let lines = vec![
            (
                "User@Host".to_string(),
                "verylonguser@verylonghostname.example.com".to_string(),
            ),
            ("OS".to_string(), "linux".to_string()),
        ];
        let user_host_width = measure_text_width(&lines[0].1);
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
            ("User@Host".to_string(), "u@h".to_string()),
            ("LongLabel".to_string(), "shortvalue".to_string()),
        ];
        let layout = compute_box_layout(&lines);
        // max_label = 9 ("LongLabel"), max_value = 10 ("shortvalue")
        assert_eq!(layout.box_width, 9 + 10 + BOX_OVERHEAD);
        assert_eq!(layout.max_value_width, 10);
    }

    #[test]
    fn box_layout_empty_input() {
        let lines: Vec<(String, String)> = vec![];
        let layout = compute_box_layout(&lines);
        // No rows: width is just the BOX_OVERHEAD (zero label + zero value).
        assert_eq!(layout.box_width, BOX_OVERHEAD);
        assert_eq!(layout.max_value_width, 0);
    }
}
