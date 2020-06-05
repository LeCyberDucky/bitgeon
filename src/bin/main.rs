use crossbeam_channel;
use std::thread;

mod state_machine;
use state_machine::State;
use state_machine::StateMachine;
mod ui;
mod util;

fn main() {
    // Initialize state machine
    let (app_tx, ui_rx) = crossbeam_channel::unbounded();
    let (ui_tx, app_rx) = crossbeam_channel::unbounded();
    let mut application = StateMachine {
        secret_key: String::from("Swordfish"),
        state: State(StateMachine::init),
        ui: util::Channel {
            sender: app_tx,
            receiver: app_rx,
        },
    };

    // Setup UI
    let ui = thread::spawn(move || {
        ui::UI::run(util::Channel {
            sender: ui_tx,
            receiver: ui_rx,
        });
    });

    application.run();

    ui.join().unwrap();
}
