// Looks like we're going https://github.com/fdehau/tui-rs 


use console::Term;


mod state_machine;
use state_machine::StateMachine;
use state_machine::State;

fn main() {
    // Initialize state machine
    let mut application = StateMachine {
        secret_key: String::from("Swordfish"),
        terminal: Term::stdout(),
        state: State(StateMachine::init),
    };

    // Run state machine
    while application.state != State(StateMachine::exit) {
        application.run();
    }
}
