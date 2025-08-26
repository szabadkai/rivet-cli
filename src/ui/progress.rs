use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("▕{bar:25}▏ {percent:>3}% • {pos}/{len} • eta {eta}")
            .expect("Invalid progress template")
            .progress_chars("█░ ")
    );
    pb
}

pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋","⠙","⠚","⠞","⠖","⠦","⠴","⠲","⠳","⠓"])
            .template("{spinner} {wide_msg}")
            .expect("Invalid spinner template")
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(message.to_string());
    pb
}