pub struct Bounce {
    pub enabled: bool,
    pub asked: bool,
}

impl Default for Bounce {
    fn default() -> Self {
        Self {
            enabled: true,
            asked: false,
        }
    }
}
