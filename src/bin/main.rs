// https://stackoverflow.com/questions/42435723/rust-tcplistener-does-not-response-outside-request
// https://www.google.dk/search?q=rust+public+tcp&ie=UTF-8&oe=
// https://doc.rust-lang.org/std/net/struct.TcpStream.html

// TODO: Consider Flume vs. crossbeam_channel https://crates.io/crates/flume

use anyhow::Result;
use crossbeam_channel;
use std::thread;
use std::time;

mod error;
mod file_processing;
mod logic_state_machine;
use logic_state_machine::LogicStateMachine;
use logic_state_machine::State;
mod settings;
mod transmission;
// use crate::transmission;
mod ui;
use ui::{StyledFilePath, StyledPathList};
mod util;

fn main() -> Result<()> {
    // Initialize state machine
    let (ui, app) = util::ThreadChannel::new_pair();
    let mut application = LogicStateMachine {
        secret_key: String::from("Swordfish"),
        state: State(LogicStateMachine::init),
        clock: time::Instant::now(),
        frame_count: 0,
        ui,
        settings: settings::Settings {
            interface_refresh_rate: 60,
            progress_refresh_rate: 4,
            internal_logic_refresh_rate: 60,
        },
        files_for_transmission: StyledPathList::new(
            String::from("Edit paths below, or simply drag and drop files or directories here:"),
            vec![StyledFilePath::new("")],
        ),
        server: transmission::Server::new(),
    };

    // Setup UI
    let ui = thread::Builder::new()
        .name("User Interface".to_string())
        .spawn(move || -> Result<()> {
            ui::UI::run( app
        )?;
            Ok(())
        })?;

    application.run()?;

    ui.join().unwrap()?;
    Ok(())
}
