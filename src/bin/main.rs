// https://stackoverflow.com/questions/42435723/rust-tcplistener-does-not-response-outside-request
// https://www.google.dk/search?q=rust+public+tcp&ie=UTF-8&oe=
// https://doc.rust-lang.org/std/net/struct.TcpStream.html

// TODO: Consider Flume vs. crossbeam_channel https://crates.io/crates/flume

use anyhow::Result;
use std::thread;

use bitgeon::application_logic::Application;
use bitgeon::server;
use bitgeon::ui;
use bitgeon::util;

fn main() -> Result<()> {
    // Initialize state machine
    let (app_to_ui, ui_to_app) = util::ThreadChannel::new_pair();
    let (app_to_server, server_to_app) = util::ThreadChannel::new_pair();
    let (ui_to_server, server_to_ui) = util::ThreadChannel::new_pair();
    let mut application = Application::new(app_to_ui, app_to_server);

    // Setup UI
    let ui = thread::Builder::new()
        .name(String::from("User Interface"))
        .spawn(move || -> Result<()> {
            let mut ui = ui::Ui::new(ui_to_app, ui_to_server);
            ui.run()?;
            Ok(())
        })?;

    let server = thread::Builder::new()
        .name(String::from("Server"))
        .spawn(move || -> Result<()> {
            let mut server = server::Server::new(server_to_app, server_to_ui);
            server.run()?;
            Ok(())
        })?;

    application.run()?;

    ui.join().unwrap()?;
    Ok(())
}
