use std::ops::Deref;
use std::time;

use anyhow::Result;

use crate::server;
use crate::settings::LogicSettings;
use crate::ui::{self, AppState};
use crate::util;
use crate::widget::{StyledFilePath, StyledPathList};

pub trait Data {}
pub trait Event {}

pub enum Message<D, E>
where
    // Bounds probably not necessary, but I want to using them. Also, they might make things look a bit nicer
    D: Data,
    E: Event,
{
    Data(D),
    Event(E),
}

pub mod data {
    use super::*;

    pub enum Server {}

    pub enum Ui {
        FilePathList(StyledPathList),
    }

    impl Data for Server {}
    impl Data for Ui {}
}

pub mod event {
    use super::*;

    pub enum Server {}

    pub enum Ui {
        Selection(usize),
    }

    impl Event for Server {}
    impl Event for Ui {}
}

pub struct State(pub fn(&mut Application) -> Result<State>);

// Used for comparing states
impl PartialEq for State {
    fn eq(&self, rhs: &Self) -> bool {
        std::ptr::eq(
            self.0 as *const fn(&mut Application) -> State,
            rhs.0 as *const fn(&mut Application) -> State,
        )
    }
}

// Without this, transitions would have this zero thing: state = state.0(&mut machine);
impl Deref for State {
    type Target = fn(&mut Application) -> Result<State>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Application {
    pub state: State,
    pub clock: time::Instant,
    pub frame_count: u128,
    pub ui: util::ThreadChannel<
        ui::Message<ui::data::Backend, ui::event::Backend>,
        Message<data::Ui, event::Ui>,
    >,
    pub settings: LogicSettings,
    pub files_for_transmission: StyledPathList,
    pub server: util::ThreadChannel<
        server::Message<server::data::Backend, server::event::Backend>,
        Message<data::Server, event::Server>,
    >,
}

impl Application {
    pub fn new(
        ui: util::ThreadChannel<
            ui::Message<ui::data::Backend, ui::event::Backend>,
            Message<data::Ui, event::Ui>,
        >,
        server: util::ThreadChannel<
            server::Message<server::data::Backend, server::event::Backend>,
            Message<data::Server, event::Server>,
        >,
    ) -> Self {
        Self {
            state: State(Application::init),
            clock: time::Instant::now(),
            frame_count: 0,
            server,
            ui,
            settings: LogicSettings::default(),
            files_for_transmission: StyledPathList::new(
                String::from(
                    "Edit paths below, or simply drag and drop files or directories here:",
                ),
                vec![StyledFilePath::new("")],
            ),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        while self.state != State(Application::exit) {
            self.state = (self.state)(self)?;
        }
        Ok(())
    }

    pub fn wait_for_input(&mut self) -> Vec<Message<data::Ui, event::Ui>> {
        let mut ui_updates = self.ui.receive();
        while ui_updates.is_empty() {
            util::sleep_remaining_frame(
                &self.clock,
                &mut self.frame_count,
                self.settings.internal_logic_refresh_rate,
            );

            ui_updates = self.ui.receive();
        }

        ui_updates
    }

    pub fn edit_files(&mut self) -> Result<State> {
        self.ui
            .send(ui::Message::Event(ui::event::Backend::StateChange(
                AppState::EditFiles(self.files_for_transmission.clone()),
            )))?;

        let ui_updates = self.wait_for_input();

        for message in ui_updates {
            match message {
                Message::Data(ui_data) => {
                    let data::Ui::FilePathList(file_paths) = ui_data;
                    self.files_for_transmission = file_paths;
                }
                Message::Event(_) => todo!(),
            }
        }

        Ok(State(Self::home))
    }

    pub fn end(&mut self) -> Result<State> {
        self.ui
            .send(ui::Message::Event(ui::event::Backend::StateChange(
                AppState::End,
            )))?;
        Ok(State(Self::exit))
    }

    pub fn exit(&mut self) -> Result<State> {
        Ok(State(Self::exit))
    }

    pub fn home(&mut self) -> Result<State> {
        self.ui
            .send(ui::Message::Event(ui::event::Backend::StateChange(
                AppState::Home(String::from("")),
            )))?;
        // self.ui
        //     .send(ui::Message::Event(ui::Event::StateChange(AppState::Home({
        //         let ip = self.server.public_ip.to_string();
        //         let port = self.server.external_port.to_string();
        //         format!("{}:{}", ip, port)
        //     }))))?;

        let ui_updates = self.wait_for_input();

        for message in ui_updates {
            match message {
                // ui::Message::Event(event) => match event {
                //     ui::Event::Selection(selection) => match selection {
                Message::Event(event::Ui::Selection(selection)) => match selection {
                    0 => return Ok(State(Self::edit_files)),
                    1 => return Ok(State(Self::receive)),
                    2 => return Ok(State(Self::end)),
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }

        Ok(State(Self::end))
    }

    pub fn init(&mut self) -> Result<State> {
        // TODO: Is this state necessary, or should we start right to home?
        // Starting is probably quick enough that we can go straight to home. If that weren't the case, this could be used for displaying start-up information or a splash screen

        Ok(State(Self::home))
    }

    pub fn receive(&mut self) -> Result<State> {
        todo!();
        Ok(State(Self::home))
    }
}
