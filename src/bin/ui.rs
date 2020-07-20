// TODO: UI should also be state machine. Or should it? Pseudo state machine maybe.
use crossterm::{self, event, ExecutableCommand};

use std::io;
use std::time;

use tui::{
    self,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListState, StatefulWidget, Text, Widget},
    style
};

use crate::util;

pub enum UIMessage {
    Element(UIElement),
    Event(UIEvent),
}

pub enum UIElement {
    Menu(UIMenu),
    Info,
    Sending,
    Receiving,
}

pub struct UIMenu {
    pub heading: String,
    pub options: Vec<String>,
    pub state: ListState,
}

impl UIMenu {
    pub fn new(heading: String, options: Vec<String>) -> UIMenu {
        UIMenu {
            heading,
            options,
            state: ListState::default(),
        }
    }

    pub fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.options.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i as isize - 1).rem_euclid(self.options.len() as isize) as usize,
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub enum UIEvent {
    Selection(usize),
    StateChange(AppState),
}

#[derive(PartialEq)]
pub enum AppState {
    Active,
    End,
}
#[derive(PartialEq, Copy, Clone)]
pub enum InterfaceState {
    MainMenu,
    SettingsEdit,
    Upload,
    End,
}

pub struct UI {
    pub application: util::Channel<UIMessage>,
    pub application_state: AppState,
    pub interface_state: InterfaceState,
    pub ui_refresh_rate: u16,
    pub clock: time::Instant,
    pub frame_count: u64,
    pub last_frame: time::Instant,
    pub frame_changed: bool,

    pub menu: UIMenu,
    pub info: UIElement,
    pub sending: UIElement,
    pub receiving: UIElement,
}

impl UI {
    pub fn run(application: util::Channel<UIMessage>) {
        // Setup
        let mut ui = UI {
            application,
            application_state: AppState::Active,
            interface_state: InterfaceState::MainMenu,
            ui_refresh_rate: 60,
            clock: time::Instant::now(),
            frame_count: 0,
            last_frame: time::Instant::now(),
            frame_changed: false,

            menu: UIMenu::new(String::new(), vec![]),
            info: UIElement::Info,
            sending: UIElement::Sending,
            receiving: UIElement::Receiving,
        };

        crossterm::terminal::enable_raw_mode();
        io::stdout().execute(crossterm::terminal::EnterAlternateScreen);
        io::stdout().execute(crossterm::cursor::Hide);
        let mut terminal = tui::Terminal::new(CrosstermBackend::new(io::stdout())).unwrap();

        while ui.interface_state != InterfaceState::End {
            ui.update();
            if ui.frame_elapsed() {
                ui.frame_count += 1;
                if ui.frame_changed { // Visual change necessitates redraw
                    ui.frame_changed = false;
                    ui.draw(&mut terminal);
                }
                ui.interact(); // User interaction
            }
        }

        // Reset terminal to initial state
        crossterm::terminal::disable_raw_mode();
        io::stdout().execute(crossterm::terminal::LeaveAlternateScreen);
        io::stdout().execute(crossterm::cursor::Show);
    }

    pub fn draw(&mut self, terminal: &mut tui::Terminal<CrosstermBackend<io::Stdout>>) {
        terminal
            .draw(|mut f| {
                // UI sections
                let split_horizontal = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                    .split(f.size());

                let split_vertical = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                    .split(split_horizontal[0]);

                let split_horizontal_1 = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(split_vertical[1]);

                // Widgets
                // let menu_frame = Block::default().title("Menu").borders(Borders::ALL);

                let style = style::Style::default();

                let menu = List::new(self.menu.options.iter().map(Text::raw))
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .style(style).highlight_style(style.fg(style::Color::Rgb(253, 3, 166)).modifier(style::Modifier::BOLD)).highlight_symbol("> ");
                // f.render_widget(menu_frame, split_vertical[0]);
                f.render_stateful_widget(menu, split_vertical[0], &mut self.menu.state);

                let info = Block::default().title("Info").borders(Borders::ALL);
                f.render_widget(info, split_horizontal[1]);

                let sending = Block::default().title("Sending").borders(Borders::ALL);
                f.render_widget(sending, split_horizontal_1[0]);

                let receiving = Block::default().title("Receiving").borders(Borders::ALL);
                f.render_widget(receiving, split_horizontal_1[1]);
            })
            .unwrap();
    }

    pub fn interact(&mut self) {
        if event::poll(time::Duration::from_secs(0)).unwrap() {
            match event::read().unwrap() {
                event::Event::Key(event) => match event.code {
                    event::KeyCode::Up => {
                        self.frame_changed = true;
                        self.menu.select_prev()
                    }, // TODO: Whether this makes sense might depend on the state of the app. Not sure if I like this
                    event::KeyCode::Down => {
                        self.frame_changed = true;
                        self.menu.select_next()
                    }, // TODO: What if we have no options? 
                    event::KeyCode::Enter => {
                        self.frame_changed = true;
                        match self.menu.state.selected() {
                            Some(option) => {self.application.send(UIMessage::Event(UIEvent::Selection(option)));},
                            None => (),
                        }
                    },
                    event::KeyCode::Esc => todo!(),
                    _ => todo!(),
                },
                // TODO: Not everything should be sent to the application thread.
                // event::Event::Key(event) => self.application.send(UIMessage::Event(UIEvent::KeyPress(event))).unwrap(),
                // _ => todo!(),
                // _ => todo!()
                _ => (),
            }
        }
    }

    // pub fn period_elapsed(&self, count: &u64, rate: &u16) -> bool {
    //     self.clock.elapsed().as_micros() >= *count as u128 * 1000 / *rate as u128 // Should we be using floating point values here? 
    // }

    pub fn frame_elapsed(&self) -> bool {
        // self.period_elapsed(&self.frame_count, &self.ui_refresh_rate)
        util::period_elapsed(&self.clock, &self.frame_count, &self.ui_refresh_rate)
    }

    pub fn update(&mut self) {
        // Get updates from logic of the program. Progress for progress bars for example
        let app_updates = self.application.receive();

        if app_updates.len() > 0 {
            for message in app_updates {
                match message {
                    UIMessage::Element(element) => match element {
                        UIElement::Menu(menu) => self.menu = menu,
                        UIElement::Receiving => todo!(),
                        UIElement::Sending => todo!(),
                        UIElement::Info => todo!(),
                    },
                    UIMessage::Event(event) => match event {
                        UIEvent::StateChange(state) => self.application_state = state,
                        UIEvent::Selection(option) => unimplemented!(),
                    },
                }
            }

            self.interface_state = match self.application_state {
                AppState::Active => self.interface_state,
                AppState::End => InterfaceState::End,
            };

            self.frame_changed = true;
        }
    }
}
