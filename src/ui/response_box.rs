use owo_colors::OwoColorize;
use reqwest::header::HeaderMap;
use std::time::Duration;

pub fn print_response_box(status: &str, duration: Duration, body_size: usize, headers: &HeaderMap) {
    // Calculate optimal width based on content
    let status_line = format!(
        "{} • {}ms • {}",
        status,
        duration.as_millis(),
        format_size(body_size)
    );
    let mut max_width = status_line.len() + 4; // Add padding

    // Check header lengths
    for (name, value) in headers {
        if let Ok(value_str) = value.to_str() {
            let header_line = format!("{}: {}", name.as_str(), value_str);
            max_width = max_width.max(header_line.len() + 4);
        }
    }

    // Ensure minimum width and reasonable maximum
    max_width = max_width.clamp(50, 100);

    // Create the top border with title
    let title = " Response ";
    let title_padding = (max_width - title.len() - 2) / 2;
    let remaining_padding = max_width - title.len() - 2 - title_padding;

    println!(
        "╭─{}{}{}─╮",
        "─".repeat(title_padding),
        title,
        "─".repeat(remaining_padding)
    );

    // Status line with colors
    let status_colored = if status.starts_with("2") {
        status.green().to_string()
    } else if status.starts_with("4") || status.starts_with("5") {
        status.red().to_string()
    } else {
        status.yellow().to_string()
    };

    let duration_ms_str = format!("{}ms", duration.as_millis());
    let size_str_plain = format_size(body_size);
    let duration_ms = duration_ms_str.dimmed();
    let size_str = size_str_plain.dimmed();
    let status_display = format!("{} • {} • {}", status_colored, duration_ms, size_str);
    let status_plain = format!("{} • {} • {}", status, duration.as_millis(), size_str_plain);
    let padding = if max_width > status_plain.len() + 2 {
        max_width - status_plain.len() - 2
    } else {
        0
    };

    println!("│ {}{} │", status_display, " ".repeat(padding));

    // Header lines
    for (name, value) in headers {
        if let Ok(value_str) = value.to_str() {
            let header_name_str = name.as_str();
            let header_name = header_name_str.dimmed();
            let header_line = format!("{}: {}", header_name, value_str);
            let plain_line = format!("{}: {}", name.as_str(), value_str);

            // Truncate if too long
            if plain_line.len() > max_width - 4 {
                let name_str = name.as_str();
                let truncated_value =
                    &value_str[..value_str.len().min(max_width - name_str.len() - 8)];
                let truncated_display = format!("{}: {}...", header_name, truncated_value);
                let truncated_plain_len = name_str.len() + truncated_value.len() + 5; // ":" + "..."
                let padding = if max_width > truncated_plain_len + 2 {
                    max_width - truncated_plain_len - 2
                } else {
                    0
                };
                println!("│ {}{} │", truncated_display, " ".repeat(padding));
            } else {
                let padding = if max_width > plain_line.len() + 2 {
                    max_width - plain_line.len() - 2
                } else {
                    0
                };
                println!("│ {}{} │", header_line, " ".repeat(padding));
            }
        }
    }

    // Bottom border
    println!("╰{}╯", "─".repeat(max_width));
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
