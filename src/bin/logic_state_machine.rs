use std::ops::Deref;
use std::time;

use anyhow::Result;

use crate::settings;
use crate::transmission;
use crate::ui::{self, AppState, Data};
use crate::util;
use crate::widget::StyledPathList;

pub struct State(pub fn(&mut LogicStateMachine) -> Result<State>);

// Used for comparing states
impl PartialEq for State {
    fn eq(&self, rhs: &Self) -> bool {
        self.0 as *const fn(&mut LogicStateMachine) -> State
            == rhs.0 as *const fn(&mut LogicStateMachine) -> State
    }
}

// Without this, transitions would have this zero thing: state = state.0(&mut machine);
impl Deref for State {
    type Target = fn(&mut LogicStateMachine) -> Result<State>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct LogicStateMachine {
    pub secret_key: String,
    pub state: State,
    pub clock: time::Instant,
    pub frame_count: u128,
    pub ui: util::ThreadChannel<ui::Message>,
    pub settings: settings::Settings,
    pub files_for_transmission: StyledPathList,
    pub server: transmission::Server,
}

impl LogicStateMachine {
    pub fn run(&mut self) -> Result<()> {
        while self.state != State(LogicStateMachine::exit) {
            self.state = (self.state)(self)?;
        }
        Ok(())
    }

    pub fn wait_for_input(&mut self) -> Vec<ui::Message> {
        let mut ui_updates;
        loop {
            // interact with ui
            ui_updates = self.ui.receive();
            if ui_updates.len() > 0 {
                break;
            }

            util::sleep_remaining_frame(
                &self.clock,
                &mut self.frame_count,
                self.settings.internal_logic_refresh_rate,
            );
        }
        ui_updates
    }

    pub fn edit_files(&mut self) -> Result<State> {
        self.ui.send(ui::Message::Event(ui::Event::StateChange(
            AppState::EditFiles(self.files_for_transmission.clone()),
        )))?;

        let ui_updates = self.wait_for_input();

        for message in ui_updates {
            match message {
                ui::Message::Data(ui_data) => {
                    let Data::FilePathList(file_paths) = ui_data;
                    self.files_for_transmission = file_paths;
                }
                ui::Message::Event(_) => todo!(),
            }
        }

        Ok(State(Self::home))
    }

    pub fn end(&mut self) -> Result<State> {
        self.ui
            .send(ui::Message::Event(ui::Event::StateChange(AppState::End)))?;
        Ok(State(Self::exit))
    }

    pub fn exit(&mut self) -> Result<State> {
        Ok(State(Self::exit))
    }

    pub fn home(&mut self) -> Result<State> {
        // self.ui
        //     .send(ui::Message::Event(ui::Event::StateChange(AppState::Home({
        //         let ip = self.server.public_ip.to_string();
        //         let port = self.server.external_port.to_string();
        //         format!("{}:{}", ip, port)
        //     }))))?;

        let ui_updates = self.wait_for_input();

        for message in ui_updates {
            match message {
                ui::Message::Event(event) => match event {
                    ui::Event::Selection(selection) => match selection {
                        0 => return Ok(State(Self::edit_files)),
                        1 => return Ok(State(Self::receive)),
                        2 => return Ok(State(Self::end)),
                        _ => todo!(),
                    },
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }

        Ok(State(Self::end))
    }

    pub fn init(&mut self) -> Result<State> {
        // Setup TCP listener
        Ok(State(Self::home))
    }

    pub fn receive(&mut self) -> Result<State> {
        todo!();
        Ok(State(Self::home))
    }
}
