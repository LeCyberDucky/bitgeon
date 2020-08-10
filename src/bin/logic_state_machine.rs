use std::ops::Deref;
use std::time;

use tui::{self, widgets::ListState};

use crate::settings;
use crate::ui::{AppState, UIEvent, UIMessage};
use crate::util;

pub struct State(pub fn(&mut LogicStateMachine) -> State);

// Used for comparing states
impl PartialEq for State {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 as *const fn(&mut LogicStateMachine) -> State
            == rhs.0 as *const fn(&mut LogicStateMachine) -> State
    }
}

// Without this, transitions would have this zero thing: state = state.0(&mut machine);
impl Deref for State {
    type Target = fn(&mut LogicStateMachine) -> State;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct LogicStateMachine {
    pub secret_key: String,
    pub state: State,
    pub clock: time::Instant,
    pub frame_count: u64,
    pub ui: util::Channel<UIMessage>,
    pub settings: settings::Settings,
}

impl LogicStateMachine {
    pub fn run(&mut self) {
        while self.state != State(LogicStateMachine::exit) {
            self.state = (self.state)(self);
        }
    }

    pub fn add_files(&mut self) -> State {
        State(Self::send)
    }

    pub fn end(&mut self) -> State {
        self.ui
            .send(UIMessage::Event(UIEvent::StateChange(AppState::End)));
        State(Self::exit)
    }

    pub fn exit(&mut self) -> State {
        State(Self::exit)
    }

    pub fn home(&mut self) -> State {
        // let ui_elements = vec![UIElement::Menu(UIMenu::new(
        //     String::from("Choose an option:"),
        //     vec![
        //         String::from("Send"),
        //         String::from("Receive"),
        //         String::from("End"),
        //     ],
        // ))];

        // for element in ui_elements {
        //     self.ui.send(UIMessage::Element(element));
        // }
        self.ui.send(UIMessage::Event(UIEvent::StateChange(AppState::Home)));

        loop {
            // interact with ui
            let ui_updates = self.ui.receive();

            // TODO: Create function for interacting since this needs to be repeated in multiple states. Let it update the frame count
            for message in ui_updates {
                match message {
                    UIMessage::Event(event) => match event {
                        UIEvent::Selection(option) => match option {
                            2 => return State(Self::end),
                            _ => todo!(),
                        },
                        UIEvent::StateChange(state) => todo!(),
                    },

                    // UIMessage::Element(element) => todo!(),
                }
            }

            util::sleep_remaining_frame(
                &self.clock,
                &self.frame_count,
                &self.settings.internal_logic_refresh_rate,
            );
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
