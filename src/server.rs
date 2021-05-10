// https://github.com/ctz/rustls

use std::collections::VecDeque;
use std::convert::TryInto;
use std::net::{IpAddr, SocketAddrV4, TcpListener, TcpStream, UdpSocket};
use std::time;

use anyhow::{anyhow, Context, Result};
use thiserror::Error;

use crate::backend;
use crate::settings::ServerSettings;
use crate::ui;
use crate::util;

// TODO: Figure out how this error handling should actually be done
#[derive(Debug, Error)]
pub enum ServerStatus {
    #[error("All good")]
    Ok,
    #[error("Unable to obtain internal port")]
    InternalPortError(anyhow::Error),
    #[error("Unable to obtain external port")]
    ExternalPortError(anyhow::Error),
    #[error("Unable to get local IP")]
    LocalIpError(anyhow::Error),
    #[error("Unable to get external IP")]
    PublicIpError(anyhow::Error),
    #[error("Unable to create TCPListener")]
    TcpBindError(anyhow::Error),
}

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

    pub enum Backend {}

    pub enum Ui {}

    impl Data for Backend {}
    impl Data for Ui {}
}

pub mod event {
    use super::*;

    pub enum Backend {}

    pub enum Ui {}

    impl Event for Backend {}
    impl Event for Ui {}
}

struct Job {}

pub struct Server {
    listener: Option<TcpListener>,
    local_ip: Option<IpAddr>,
    public_ip: Option<IpAddr>,
    internal_port: Option<u16>,
    external_port: Option<u16>,
    upnp_lease_clock: time::Instant,
    upnp_lease_duration: time::Duration,
    peers: Vec<Peer>,
    status: ServerStatus, // Should we have a vector of errors or only store one at a time?
    application: util::ThreadChannel<
        backend::Message<backend::data::Server, backend::event::Server>,
        Message<data::Backend, event::Backend>,
    >,
    ui: util::ThreadChannel<
        ui::Message<ui::data::Server, ui::event::Server>,
        Message<data::Ui, event::Ui>,
    >,
    secret_key: String,
    clock: time::Instant,
    queue: VecDeque<Job>,
    frame_count: u128,
    settings: ServerSettings,
}

impl Server {
    pub fn new(
        application: util::ThreadChannel<
            backend::Message<backend::data::Server, backend::event::Server>,
            Message<data::Backend, event::Backend>,
        >,
        ui: util::ThreadChannel<
            ui::Message<ui::data::Server, ui::event::Server>,
            Message<data::Ui, event::Ui>,
        >,
    ) -> Self {
        Self {
            listener: None,
            local_ip: None,
            public_ip: None,
            internal_port: None,
            external_port: None,
            upnp_lease_clock: time::Instant::now(),
            upnp_lease_duration: time::Duration::from_secs(60 * 15), // TODO: Don't hard code this. Read from config file but also provide default value
            peers: vec![],
            status: ServerStatus::Ok,
            application,
            ui,
            secret_key: String::from("Swordfish"),
            clock: time::Instant::now(),
            queue: VecDeque::new(),
            frame_count: 0,
            settings: ServerSettings::default(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Initialize server
        self.refresh_connection();
        self.display_connection();

        loop {
            // Listen for incoming connections
            // Listen for incoming messages from peers
            // Listen for messages from application or UI
            // Work on jobs
            // Sleep

            util::sleep_remaining_frame(
                &self.clock,
                &mut self.frame_count,
                self.settings.logic_refresh_rate,
            );
        }

        Ok(())
    }

    pub fn refresh_connection(&mut self) {
        self.status = ServerStatus::Ok;

        // Bind listener to every available interface. Let OS provide an available port.
        // TODO: Perhaps let the user specify a port on their own
        match TcpListener::bind("0.0.0.0:0") {
            Ok(listener) => self.listener = Some(listener),
            Err(error) => {
                self.status = ServerStatus::TcpBindError(anyhow!(error));
                return;
            }
        }

        match get_local_ip() {
            Ok(ip) => self.local_ip = Some(ip),
            Err(error) => {
                self.status = ServerStatus::LocalIpError(anyhow!(error));
                return;
            }
        };

        match get_public_ip() {
            Ok(ip) => self.public_ip = Some(ip),
            Err(error) => {
                self.status = ServerStatus::PublicIpError(anyhow!(error));
                return;
            }
        };

        // This unwrap will only be reached if the listener is ok above
        match self.listener.as_ref().unwrap().local_addr() {
            Ok(address) => self.internal_port = Some(address.port()),
            Err(error) => {
                self.status = ServerStatus::InternalPortError(anyhow!(error));
                return;
            }
        };

        match self.add_port_mapping() {
            Ok(port) => self.external_port = Some(port),
            Err(error) => {
                self.status = ServerStatus::ExternalPortError(anyhow!(error));
                return;
            }
        }
    }

    pub fn accept_connection(&mut self) -> Result<Option<Peer>> {
        // // Should we rely on the existent data (local_ip, public_ip), or should we update it?
        // // I guess updating doesn't hurt and might be more reliable, in case stuff has changed.

        // // 1. Update data
        // // TODO: If we can't get a local or public IP, we shouldn't crash, but the user should be notified.
        // self.refresh_ips()
        //     .with_context(|| format!("Unable to listen for connection."))?;
        // if self.upnp_lease_clock.elapsed() >= self.upnp_lease_duration {
        //     self.add_port_mapping()
        //         .with_context(|| format!("Unable to listen for connection."))?;
        // }

        // // self.local_ip = get_local_ip().ok();
        // // self.public_ip = get_public_ip().ok();

        // // 2. Forward port via UPnP to allow for incoming connection
        todo!();
    }

    pub fn establish_connection() -> Result<Option<Peer>> {
        // Open outgoing connection to peer
        todo!();
    }

    // fn get_free_port() -> Result<u16> {
    //     // Binding the listener to port 0 will request an unused port from the OS
    //     let listener = TcpListener::bind("0.0.0.0:0")?;
    //     let local_address = listener.local_addr()?;
    //     Ok(local_address.port())
    // }

    pub fn add_port_mapping(&mut self) -> Result<u16> {
        // Add port mapping to gateway device via UPnP
        // let local_ip = self.local_ip.ok_or_else( || "No local IP address.")?;
        let internal_port = self
            .internal_port
            .with_context(|| String::from("No internal port."))?;
        let local_ip = self
            .local_ip
            .with_context(|| String::from("No local IP address."))?;
        let local_ip = match local_ip {
            IpAddr::V4(ipv4) => Ok(ipv4),
            // IpAddr::V6(ipv6) => Err("Local IP is IPv6, but only IPv4 is supported.".into()),
            IpAddr::V6(_) => Err(anyhow!(String::from(
                "Local IP is IPv6, but only IPv4 is supported."
            ))),
        }?;
        let local_address = SocketAddrV4::new(local_ip, internal_port);
        let gateway = igd::search_gateway(Default::default())
            .with_context(|| String::from("Unable to find gateway device. Verify connection."))?;
        let external_port = gateway
            .add_any_port(
                igd::PortMappingProtocol::TCP,
                local_address,
                self.upnp_lease_duration
                    .as_secs()
                    .try_into()
                    .with_context(|| String::from("UPnP lease duration should fit into u32"))?,
                "Bitgeon",
            )
            .with_context(|| String::from("Unable to add port mapping via UPnP."))?;
        self.upnp_lease_clock = time::Instant::now();
        self.external_port = Some(external_port);
        Ok(external_port)
    }

    pub fn refresh_ips(&mut self) -> Result<()> {
        // Refresh connection info. Call this in case of errors trying to establish a connection, but also let the user call this.
        let local_ip =
            get_local_ip().with_context(|| String::from("Unable to get local IP address."))?;
        self.local_ip = Some(local_ip);
        let public_ip =
            get_public_ip().with_context(|| String::from("Unable to get public IP address."))?;
        self.public_ip = Some(public_ip);
        Ok(())
    }

    pub fn display_connection(&self) -> Result<()> {
        todo!();
        // let message;

        // self.ui.send(message)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Peer {
    tcp_stream: TcpStream,
}

pub fn get_public_ip() -> Result<IpAddr> {
    let gateway = igd::search_gateway(Default::default()).with_context(|| {
        String::from("Unable to find gateway. Verify your internet connection.")
    })?;

    let ip = gateway
        .get_external_ip()
        .with_context(|| String::from("Unable to get external IP address."))?;
    Ok(IpAddr::from(ip))
}

pub fn get_local_ip() -> Result<IpAddr> {
    // https://stackoverflow.com/a/166589/5780938

    let gateway = igd::search_gateway(Default::default()).with_context(|| {
        String::from("Unable to find gateway. Verify your internet connection.")
    })?;

    // Bind to every available interface on an unused port
    let socket = UdpSocket::bind("0.0.0.0:0")
        .with_context(|| String::from("Unable to bind UDP socket to \"0.0.0.0:0\"."))?;

    socket
        .connect(gateway.addr)
        .with_context(|| String::from("Unable to connect to gateway."))?;

    let local_address = socket.local_addr()?;
    Ok(local_address.ip())
}
