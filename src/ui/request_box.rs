use owo_colors::OwoColorize;

pub fn print_request_box(method: &str, url: &str, headers: &[String]) {
    // Calculate the optimal box width based on content
    let method_url_line = format!("{} {}", method, url);
    let mut max_width = method_url_line.len() + 4; // Add padding

    // Check header lengths
    for header in headers {
        let display_header = if header.to_lowercase().contains("authorization") {
            let parts: Vec<&str> = header.splitn(2, ':').collect();
            if parts.len() == 2 {
                format!("{}: ****", parts[0].trim())
            } else {
                header.clone()
            }
        } else {
            header.clone()
        };
        max_width = max_width.max(display_header.len() + 4);
    }

    // Ensure minimum width and reasonable maximum
    max_width = max_width.clamp(50, 100);

    // Create the top border
    let title = " Request ";
    let title_padding = (max_width - title.len() - 2) / 2;
    let remaining_padding = max_width - title.len() - 2 - title_padding;

    println!(
        "╭─{}{}{}─╮",
        "─".repeat(title_padding),
        title,
        "─".repeat(remaining_padding)
    );

    // Method and URL line
    let method_colored = method.bright_green();
    let content_line = format!("{} {}", method_colored, url);
    let content_plain = format!("{} {}", method, url);
    let padding = if max_width > content_plain.len() + 2 {
        max_width - content_plain.len() - 2
    } else {
        0
    };
    println!("│ {}{} │", content_line, " ".repeat(padding));

    // Header lines
    for header in headers {
        let display_header = if header.to_lowercase().contains("authorization") {
            let parts: Vec<&str> = header.splitn(2, ':').collect();
            if parts.len() == 2 {
                format!("{}: {}", parts[0].trim(), "****".dimmed())
            } else {
                header.clone()
            }
        } else {
            header.clone()
        };

        let plain_header = if header.to_lowercase().contains("authorization") {
            let parts: Vec<&str> = header.splitn(2, ':').collect();
            if parts.len() == 2 {
                format!("{}: ****", parts[0].trim())
            } else {
                header.clone()
            }
        } else {
            header.clone()
        };

        let padding = if max_width > plain_header.len() + 2 {
            max_width - plain_header.len() - 2
        } else {
            0
        };
        println!("│ {}{} │", display_header, " ".repeat(padding));
    }

    // Bottom border
    println!("╰{}╯", "─".repeat(max_width));
    println!();
}
