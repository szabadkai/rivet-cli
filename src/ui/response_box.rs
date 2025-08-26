use owo_colors::OwoColorize;
use reqwest::header::HeaderMap;
use std::time::Duration;

pub fn print_response_box(status: &str, duration: Duration, body_size: usize, headers: &HeaderMap) {
    println!("╭─ Response ─────────────────────────────╮");
    
    let status_colored = if status.starts_with("2") {
        status.green().to_string()
    } else if status.starts_with("4") || status.starts_with("5") {
        status.red().to_string()
    } else {
        status.yellow().to_string()
    };
    
    let duration_ms = duration.as_millis();
    let size_str = format_size(body_size);
    
    println!("│ {} • {}ms • {}            │", 
        status_colored, 
        duration_ms, 
        size_str
    );
    
    for (name, value) in headers {
        if let Ok(value_str) = value.to_str() {
            let display_line = format!("{}: {}", name.as_str(), value_str);
            if display_line.len() <= 38 {
                println!("│ {}         │", display_line);
            } else {
                println!("│ {}... │", &display_line[..35]);
            }
        }
    }
    
    println!("╰────────────────────────────────────────╯");
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