use console::Term;
use crossbeam_channel::unbounded;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::ops::Deref;
use std::thread;
use std::time::Duration;
    
pub struct State(pub fn(&mut StateMachine) -> State);

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

pub struct StateMachine {
    pub secret_key: String,

    pub state: State,
    pub terminal: Term,
}

impl StateMachine {
    pub fn run(&mut self) {
        self.state = (self.state)(self);
    }

    // #####################
    // State implementations
    // #####################

    pub fn add_files(&mut self) -> State {
        self.terminal.clear_screen();
        println!("Drop files here to make them available for transmission. Press enter when you are done:\n");
        State(Self::send)
    }

    pub fn end(&mut self) -> State {
        self.terminal.clear_screen();
        State(Self::exit)
    }

    pub fn exit(&mut self) -> State {
        State(Self::exit)
    }

    pub fn home(&mut self) -> State {
        self.terminal.clear_screen();

        let options = &["Send", "Receive", "End"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an option")
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

    pub fn init(&mut self) -> State {
        self.terminal.clear_screen();
        println!("Initializing!");
        State(Self::home)
    }

    pub fn receive(&mut self) -> State {
        println!("Receiving!");
        State(Self::end)
    }

    pub fn send(&mut self) -> State {
        // Logic for this state: 
        /* Continuously: 
         * - Show sending progress 
         * - Allow user to do stuff (Menu to go to different state [home, add files])
         *
         *
         */
        








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
