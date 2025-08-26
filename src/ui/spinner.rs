use indicatif::ProgressBar;
use crate::ui::progress::create_spinner;

pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Self {
            pb: create_spinner(message),
        }
    }
    
    pub fn set_message(&self, message: &str) {
        self.pb.set_message(message.to_string());
    }
    
    pub fn finish_with_message(&self, message: &str) {
        self.pb.finish_with_message(message.to_string());
    }
}