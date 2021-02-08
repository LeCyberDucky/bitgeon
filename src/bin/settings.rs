pub struct Settings {
    pub interface_refresh_rate: u128, // Update rate on user interaction in Hz
    pub progress_refresh_rate: u128,  // Update rate for progress bars at idle in Hz
    pub internal_logic_refresh_rate: u128,
}

mod config {}

mod style {}
