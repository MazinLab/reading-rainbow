// Nikki Zivkov 02/10/2025
// Readout status

pub struct Status {
    pub status_message: String,
}

impl Status {
    pub fn new() -> Self {
        Self {
            status_message: String::from("Disconnected"),
        }
    }

    #[allow(dead_code)]
    pub fn update(&mut self, message: &str) {
        self.status_message = message.to_string();
    }
}

// Implementing Default trait for Status
impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}