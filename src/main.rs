use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::ops::Deref;
use std::thread;
use std::time::Duration;

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


struct StateMachine {
    secret_key: String,

    state: State,
    terminal: Term,
}

impl StateMachine {
    fn run(&mut self) {
        self.state = (self.state)(self);
    }

    // #####################
    // State implementations
    // #####################
    
    fn end(&mut self) -> State {
        self.terminal.clear_screen();
        State(Self::exit)
    }
    
    fn exit(&mut self) -> State {
        State(Self::exit)
    }
    
    fn home(&mut self) -> State {
        self.terminal.clear_screen();
        
        let options = &[
            "Send",
            "Receive",
            "End",
            ];
            
        let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose an option:")
        .default(0)
        .items(&options[..])
        .interact()
        .unwrap();
        
        self.terminal.clear_screen();
        
        // TODO: Find better way for doing this. Matching against enums or something instead would be nice. 
        match selection {
            0 => State(Self::send),
            1 => State(Self::receive),
            2 => State(Self::end),
            _ => State(Self::exit),
        }
    }
    
    fn init(&mut self) -> State {
        self.terminal.clear_screen();
        println!("Initializing!");
        State(Self::home)
    }
    
    fn receive(&mut self) -> State {
        println!("Receiving!");
        State(Self::end)
    }
    
    fn send(&mut self) -> State {
        println!("Sending!");
        
        let pb = ProgressBar::new(1024);
        
        for _ in 0..1024 {
            pb.inc(1);
            thread::sleep(Duration::from_millis(5));
        }
        
        pb.finish_with_message("done");
        
        State(Self::end)
    }
}
