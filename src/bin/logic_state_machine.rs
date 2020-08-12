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

    pub fn wait_for_input(&mut self) -> Vec<UIMessage> {
        let mut ui_updates = vec![];
        loop {
            // interact with ui
            ui_updates = self.ui.receive();
            if ui_updates.len() > 0 {
                break;
            }

            util::sleep_remaining_frame(
                &self.clock,
                &mut self.frame_count,
                &self.settings.internal_logic_refresh_rate,
            );
        }
        ui_updates
    }

    pub fn add_files(&mut self) -> State {
        self.ui.send(UIMessage::Event(UIEvent::StateChange(AppState::AddFiles)));

        let mut ui_updates = self.wait_for_input();

        // for message in ui_updates {
        //     match message {
        //         UIMessage::UserInput()
        //     }
        // }

        State(Self::home)
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
        self.ui.send(UIMessage::Event(UIEvent::StateChange(AppState::Home)));

        let mut ui_updates = self.wait_for_input();

        for message in ui_updates {
            match message {
                UIMessage::Event(event) => match event {
                    UIEvent::Selection(selection) => match selection {
                        0 => return State(Self::add_files),
                        1 => return State(Self::receive),
                        2 => return State(Self::end),
                        _ => todo!(),
                    },
                    _ => todo!(),
                },
            }
        }

        State(Self::end)
    }

    pub fn init(&mut self) -> State {
        State(Self::home)
    }

    pub fn receive(&mut self) -> State {
        State(Self::home)
    }
}
