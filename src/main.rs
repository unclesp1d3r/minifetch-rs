use chrono::Local;
use clap::Parser;
use console::{measure_text_width, style, Term};
use figlet_rs::FIGfont;
use indicatif::HumanBytes;
use std::collections::HashSet;
use sysinfo::{ComponentExt, CpuExt, DiskExt, NetworkExt, System, SystemExt, UserExt};
use users::{get_current_uid, get_user_by_uid};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli;

fn main() {
    let _cli = Cli::parse();
    let term = Term::stdout();

    let mut sys = System::new_all();
    sys.refresh_all();

    let user = get_user_by_uid(get_current_uid()).expect("No user found for current UID");
    let hostname = sys.host_name().unwrap_or_else(|| "N/A".to_string());

    // Figlet for hostname
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert(&hostname);
    term.write_line(&format!("{}", style(figure.unwrap().to_string()).cyan()))
        .unwrap();

    let mut content_lines: Vec<(String, String)> = Vec::new(); // (label, value)

    // User and Hostname line (unstyled for width calculation)
    let unstyled_user_hostname = format!("{}@{}", user.name().to_str().unwrap_or("N/A"), hostname);
    content_lines.push(("User@Host".to_string(), unstyled_user_hostname));

    // OS and Kernel
    content_lines.push((
        "OS".to_string(),
        format!(
            "{} {}",
            sys.name().unwrap_or_else(|| "N/A".to_string()),
            sys.os_version().unwrap_or_else(|| "N/A".to_string())
        ),
    ));
    content_lines.push((
        "Kernel".to_string(),
        sys.kernel_version().unwrap_or_else(|| "N/A".to_string()),
    ));

    // Uptime
    let uptime =
        humantime::format_duration(std::time::Duration::from_secs(sys.uptime())).to_string();
    content_lines.push(("Uptime".to_string(), uptime));

    // Logged-in Users
    let users: Vec<String> = sys.users().iter().map(|u| u.name().to_string()).collect();
    if !users.is_empty() {
        content_lines.push(("Users".to_string(), users.join(", ")));
    }

    // CPU Utilization
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    content_lines.push((
        "CPU".to_string(),
        format!(
            "{:.2}% {}",
            cpu_usage,
            render_bar(cpu_usage as u64, 100, 20)
        ),
    ));

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
    let mut seen_disks = HashSet::new();
    for disk in sys.disks() {
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
    for (interface_name, data) in sys.networks() {
        // Filter for interfaces with activity
        if data.received() > 0 || data.transmitted() > 0 {
            content_lines.push((
                format!("Net ({})", interface_name),
                format!(
                    "{} (Rx: {}, Tx: {})",
                    data.mac_address(),
                    HumanBytes(data.received()),
                    HumanBytes(data.transmitted())
                ),
            ));
        }
    }

    // Temperatures
    let mut temp_count = 0;
    for component in sys.components() {
        // Filter out unrealistic temperatures and limit the number of displayed sensors
        if component.temperature() > 0.0 && component.temperature() < 100.0 && temp_count < 5 {
            content_lines.push((
                format!("Temp ({})", component.label()),
                format!("{:.1}°C", component.temperature()),
            ));
            temp_count += 1;
        }
    }

    // Calculate max width for content lines
    let mut max_label_width = 0;
    let mut max_value_width = 0;
    for (label, value) in &content_lines {
        let label_width = measure_text_width(label);
        let value_width = measure_text_width(value);
        if label_width > max_label_width {
            max_label_width = label_width;
        }
        if value_width > max_value_width {
            max_value_width = value_width;
        }
    }

    // Adjust for padding and box characters
    let box_width = max_label_width + max_value_width + 6; // 2 for padding, 2 for box characters, 2 for ": "

    // Print the box
    term.write_line(&format!(
        "{}{}{}",
        style("┌").black().bright(),
        "─".repeat(box_width - 2),
        style("┐").black().bright()
    ))
    .unwrap();

    // Print user@hostname line (centered)
    let user_hostname_line_content = &content_lines[0].1;
    let user_hostname_width = measure_text_width(user_hostname_line_content);
    let padding_left = (box_width - user_hostname_width - 2) / 2;
    let padding_right = box_width - user_hostname_width - 2 - padding_left;
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
    ))
    .unwrap();

    term.write_line(&format!(
        "{}{}{}",
        style("├").black().bright(),
        "─".repeat(box_width - 2),
        style("┤").black().bright()
    ))
    .unwrap();

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
        let value_padding = max_value_width - current_value_width;

        term.write_line(&format!(
            "{} {}: {}{}{}",
            style("│").black().bright(),
            style(label).cyan().bright(),
            styled_value,
            " ".repeat(value_padding),
            style("│").black().bright()
        ))
        .unwrap();
    }

    term.write_line(&format!(
        "{}{}{}",
        style("└").black().bright(),
        "─".repeat(box_width - 2),
        style("┘").black().bright()
    ))
    .unwrap();
    term.write_line(&format!(
        "{}: {}",
        style("Date").green(),
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ))
    .unwrap();
}

fn render_bar(value: u64, max: u64, length: usize) -> String {
    let filled_chars = ((value as f64 / max as f64) * length as f64).round() as usize;
    let empty_chars = length - filled_chars;
    format!("{}{}", "█".repeat(filled_chars), " ".repeat(empty_chars))
}

#[test]
fn test_width_calculation() {
    assert_eq!(measure_text_width("Hello"), 5);
    assert_eq!(measure_text_width("Hello"), 5);
    assert_eq!(
        measure_text_width(&format!("{}{}", "█".repeat(5), " ".repeat(5))),
        10
    );
}
