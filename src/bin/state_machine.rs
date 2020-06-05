use std::ops::Deref;

use tui::{self, widgets::ListState};

use crate::ui::{UIElement, UIMenu, UIMessage, UIEvent, AppState};
use crate::util;

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
    pub ui: util::Channel<UIMessage>,
}

impl StateMachine {
    pub fn run(&mut self) {
        while self.state != State(StateMachine::exit) {
            self.state = (self.state)(self);
        }
    }

    pub fn add_files(&mut self) -> State {
        State(Self::send)
    }

    pub fn end(&mut self) -> State {
        self.ui.send(UIMessage::Event(UIEvent::StateChange(AppState::End)));
        State(Self::exit)
    }

    pub fn exit(&mut self) -> State {
        State(Self::exit)
    }

    pub fn home(&mut self) -> State {
        let ui_elements = vec![UIElement::Menu(UIMenu::new(
            String::from("Choose an option:"),
            vec![
                String::from("Send"),
                String::from("Receive"),
                String::from("End"),
            ],
        ))];

        for element in ui_elements {
            self.ui.send(UIMessage::Element(element));
        }

        loop {
            // interact with ui
            let ui_updates = self.ui.receive();

            for message in ui_updates {
                match message {
                    UIMessage::Event(event) => match event {
                        UIEvent::Selection(option) => match option {
                            2 => return State(Self::end),
                            _ => todo!(),
                        }, 
                        UIEvent::StateChange(state) => todo!(),
                    },

                    UIMessage::Element(element) => todo!(),
                }
            }
        }

        // // TODO: Find better way for doing this. Matching against enums or something instead would be nice.
        // match selection {
        //     0 => State(Self::send),
        //     1 => State(Self::receive),
        //     2 => State(Self::end),
        //     _ => State(Self::exit),
        // }

        State(Self::end)
    }

    pub fn init(&mut self) -> State {
        State(Self::home)
    }

    pub fn receive(&mut self) -> State {
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
        State(Self::end)
    }
}
