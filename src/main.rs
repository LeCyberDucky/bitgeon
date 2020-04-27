// https://hoverbear.org/blog/rust-state-machine-pattern/
// https://www.reddit.com/r/rust/comments/ft1hqh/state_machines_in_rust/
// http://cliffle.com/blog/rust-typestate/
// https://www.reddit.com/r/rust/comments/57ccds/pretty_state_machine_patterns_in_rust/d8rhwq4/
// https://dev.to/mindflavor/lets-build-zork-using-rust-1opm


// Consider  using a struct or enum variant for each state, with each transition function being a method defined on it that takes self and returns Self.

use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use std::ops::Deref;

fn main() {
    // Initialize state machine
    let mut application = StateMachine {
        secret_key: String::from("Swordfish"),
        terminal: Term::stdout(),
    };
    let mut app_state = State(StateMachine::init);

    // Run state machine
    while app_state != State(StateMachine::exit) {
        app_state = app_state(&mut application);
    }
}


struct State(fn(&mut StateMachine) -> State);

// Used for comparing states
impl PartialEq for State {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 as *const fn(&mut StateMachine) -> State 
            == rhs.0 as *const fn(&mut StateMachine) -> State
    }
}

// Without this, transitions would have this zero thing: state = state.0(&mut machine);
impl Deref for State {
    type Target = fn(&mut StateMachine) -> State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Debug)]
struct StateMachine {
    secret_key: String,
    terminal: Term,
}

impl StateMachine {
    fn init(&mut self) -> State {
        println!("Initializing!");
        // self.terminal.write_line("Initializing!");
        State(Self::home)
    }

    fn home(&mut self) -> State {
        self.terminal.clear_screen();
        println!("Welcome!\n");

        let options = &[
            "Send",
            "Receive",
            "End",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Watcha wanna do?")
            .default(0)
            .items(&options[..])
            .interact()
            .unwrap();
        
        self.terminal.clear_screen();

        match selection {
            0 => return State(Self::send),
            1 => return State(Self::receive),
            2 => return State(Self::end),
            _ => return State(Self::exit),
        }

        State(Self::send)
    }

    fn send(&mut self) -> State {
        println!("Sending!");
        // self.terminal.write_line("Sending!");
        State(Self::end)
    }

    fn receive(&mut self) -> State {
        println!("Receiving!");
        // self.terminal.write_line("Receiving!");
        State(Self::end)
    }

    fn end(&mut self) -> State {
        println!("Bye!");
        // self.terminal.write_line("Bye!");
        State(Self::exit)
    }

    fn exit(&mut self) -> State {
        State(Self::exit)
    }
}
