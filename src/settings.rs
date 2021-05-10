pub struct LogicSettings {
    pub interface_refresh_rate: u128, // Update rate on user interaction in Hz
    pub progress_refresh_rate: u128,  // Update rate for progress bars at idle in Hz
    pub internal_logic_refresh_rate: u128,
}

impl LogicSettings {
    fn new(
        interface_refresh_rate: u128,
        progress_refresh_rate: u128,
        internal_logic_refresh_rate: u128,
    ) -> Self {
        Self {
            interface_refresh_rate,
            progress_refresh_rate,
            internal_logic_refresh_rate,
        }
    }
}

impl Default for LogicSettings {
    fn default() -> Self {
        Self::new(60, 4, 60)
    }
}

pub struct ServerSettings {
    pub logic_refresh_rate: u128,
}

impl ServerSettings {
    fn new(logic_refresh_rate: u128) -> Self {
        Self { logic_refresh_rate }
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self::new(60)
    }
}

mod config {}

mod style {}
