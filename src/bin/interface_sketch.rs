// https://qiita.com/wangya_eecs/items/b9e1a501cb0c0ab0de1c
// https://github.com/hatoo/oha/blob/master/src/monitor.rs

// UI structure
// On every update iteration, it checks for messages from the application
// The messages are enum variants, like "Menu", containing what is necessary for drawing that element of the ui. It stores the contents and keep drawing these, until they are updated via a new message

// It also sends messages to the application. These are enum variants like "Key press" or "String"

use crossterm::{self, terminal, ExecutableCommand};

use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Widget};
use tui::{self, backend::CrosstermBackend};

use std::io;

use std::thread;
use std::time::Duration;

fn main() {
    terminal::enable_raw_mode();

    // This prevents drawing over other stuff that is already there
    io::stdout().execute(crossterm::terminal::EnterAlternateScreen); // TODO: Should this be reversed when done?
    io::stdout().execute(crossterm::cursor::Hide); // TODO: Unhide cursor when done

    let mut stdout = io::stdout();
    // crossterm::execute!(stdout, terminal::EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = tui::Terminal::new(backend).unwrap();

    // terminal.clear();

    terminal
        .draw(|mut f| {
            // Draw new frame

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

            let menu = Block::default().title("Menu").borders(Borders::ALL);
            f.render_widget(menu, split_vertical[0]);

            let info = Block::default().title("Info").borders(Borders::ALL);
            f.render_widget(info, split_horizontal[1]);

            let sending = Block::default().title("Sending").borders(Borders::ALL);
            f.render_widget(sending, split_horizontal_1[0]);

            let receiving = Block::default().title("Receiving").borders(Borders::ALL);
            f.render_widget(receiving, split_horizontal_1[1]);
        })
        .unwrap();

    thread::sleep(Duration::from_millis(5000));

    terminal
        .draw(|mut f| {
            // Draw new frame

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

            let sending = Block::default().title("Bloppel").borders(Borders::ALL);
            f.render_widget(sending, split_horizontal_1[0]);
        })
        .unwrap();

    thread::sleep(Duration::from_millis(5000));

    terminal::disable_raw_mode();
    io::stdout().execute(crossterm::terminal::LeaveAlternateScreen);
    io::stdout().execute(crossterm::cursor::Show);
}

// use crossbeam_channel::unbounded;
// use crossterm::raw::IntoRawMode;
// use std::io;
// use tui::backend::CrosstermBackend;
// use tui::Terminal;

// fn main() {
//     let stdout = io::stdout();
//     let backend = CrosstermBackend::new(stdout);
//     // let mut terminal = Terminal::new(backend);

//     let mut ui = UI{
//         refresh_rate: 30,
//         terminal: Terminal::new(backend).unwrap(),
//     };
// }

// struct UI {
//     refresh_rate: u8, // Frames per second
//     terminal: Terminal<CrosstermBackend<io::Stdout>>,
// }
