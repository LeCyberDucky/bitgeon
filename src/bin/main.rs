// https://stackoverflow.com/questions/42435723/rust-tcplistener-does-not-response-outside-request
// https://www.google.dk/search?q=rust+public+tcp&ie=UTF-8&oe=
// https://doc.rust-lang.org/std/net/struct.TcpStream.html

// TODO: Consider Flume vs. crossbeam_channel https://crates.io/crates/flume

use anyhow::Result;
use std::thread;

mod application_logic;
mod file_processing;
use application_logic::Application;
mod settings;
mod transmission;
// use crate::transmission;
mod ui;
mod util;
mod widget;

fn main() -> Result<()> {
    // Initialize state machine
    let (ui, app) = util::ThreadChannel::new_pair();
    let mut application = Application::new(ui);

    // Setup UI
    let ui = thread::Builder::new()
        .name("User Interface".to_string())
        .spawn(move || -> Result<()> {
            ui::UI::run(app)?;
            Ok(())
        })?;

    application.run()?;

    ui.join().unwrap()?;
    Ok(())
}
