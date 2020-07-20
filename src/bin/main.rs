use crossbeam_channel;
use std::thread;
use std::time;

mod logic_state_machine;
use logic_state_machine::State;
use logic_state_machine::LogicStateMachine;
mod ui;
mod util;
mod settings;

fn main() {
    // Initialize state machine
    let (app_tx, ui_rx) = crossbeam_channel::unbounded();
    let (ui_tx, app_rx) = crossbeam_channel::unbounded();
    let mut application = LogicStateMachine {
        secret_key: String::from("Swordfish"),
        state: State(LogicStateMachine::init),
        clock: time::Instant::now(),
        frame_count: 0,
        ui: util::Channel {
            sender: app_tx,
            receiver: app_rx,
        },
        settings: settings::Settings {
            interface_refresh_rate: 60,
            progress_refresh_rate: 4,
            internal_logic_refresh_rate: 200,
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
