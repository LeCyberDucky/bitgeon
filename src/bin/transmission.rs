// https://github.com/ctz/rustls

use std::convert::TryInto;
use std::net::{IpAddr, SocketAddrV4, TcpListener, TcpStream, UdpSocket};
use std::time;

use anyhow::{anyhow, Context, Result};
use thiserror::Error;

// use crate::error::AppError;

#[derive(Debug, Error)]
pub enum ServerStatus {
    #[error("All good")]
    Ok,
    // Initializing,
    // GatewayError,
    #[error("Unable to obtain internal port")]
    InternalPortError(anyhow::Error),
    #[error("Unable to obtain external port")]
    ExternalPortError(anyhow::Error),
    #[error("Unable to get local IP")]
    LocalIPError(anyhow::Error),
    #[error("Unable to get external IP")]
    PublicIPError(anyhow::Error),
    #[error("Unable to create TCPListener")]
    TCPBindError(anyhow::Error),
    // UPnPError,
    // #[error(transparent)]
    // Other(#[from] anyhow::Error),
}

// impl fmt::Display for ServerStatus {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Server error. Whoopsie!") // TODO: Actually give meaningful output here
//     }
// }

pub struct Server {
    // local_address: Option<SocketAddr>,
    listener: Option<TcpListener>,
    pub local_ip: Option<IpAddr>,
    pub public_ip: Option<IpAddr>,
    pub internal_port: Option<u16>,
    pub external_port: Option<u16>,
    upnp_lease_clock: time::Instant,
    // upnp_lease_duration: u32, // UPnP lease time here only supports u32. time::Duration takes u64.
    upnp_lease_duration: time::Duration,
    peers: Vec<Peer>,
    pub status: ServerStatus, // Should we have a vector of errors or only store one at a time?
}

impl Server {
    pub fn new() -> Server {
        let mut server = Server {
            listener: None,
            local_ip: None,
            public_ip: None,
            internal_port: None,
            external_port: None,
            upnp_lease_clock: time::Instant::now(),
            // upnp_lease_duration: 60 * 15,
            upnp_lease_duration: time::Duration::from_secs(60 * 15),
            peers: vec![],
            status: ServerStatus::Ok,
        };

        server.refresh_connection();
        server

        // // No need to crash if connection doesn't work initially
        // match TcpListener::bind("0.0.0.0:0") {
        //     Ok(listener) => server.listener = Some(listner),
        //     Err(error) => {
        //         server.status = ServerStatus::from(error.into());
        //         return server;
        //     }
        // }

        // match get_local_ip() {
        //     Ok(ip) => server.local_ip = Some(ip),
        //     Err(error) => {
        //         server.status = ServerStatus::from(error.into());
        //         return server;
        //     }
        // };

        // match get_public_ip() {
        //     Ok(ip) => server.public_ip = Some(ip),
        //     Err(error) => {
        //         server.status = ServerStatus::from(error.into());
        //         return server;
        //     }
        // };

        // // This unwrap will only be reached if the listener is ok above
        // match server.listener.unwrap().local_addr() {
        //     Ok(address) => server.internal_port = address.port(),
        //     Err(error) => {
        //         server.status = ServerStatus::from(error.into());
        //         return server;
        //     }
        // };

        // match add_port_mapping() {
        //     Ok(port) => server.external_port = Some(port),
        //     Err(error) => {
        //         server.status = ServerStatus::from(error.into());
        //         return server;
        //     }
        // }

        // server

        // let local_ip = get_local_ip().ok();
        // // Bind listener to every available interface. Let OS provide an available port.
        // // TODO: Perhaps let the user specify a port on their own
        // let listener = TcpListener::bind("0.0.0.0:0").ok();
        // let internal_port = match listener {
        //     Some(listener) => match listener.local_addr() {
        //         Ok(address) => address.port(),
        //         Err(_) => None,
        //     },
        //     None => None,
        // };

        // let public_ip = get_public_ip().ok();
        // let external_port = None;

        // Server {
        //     listener,
        //     local_ip: local_ip,
        //     internal_port,
        //     public_ip: public_ip.ok(),
        //     external_port,
        //     upnp_lease_clock: time::Instant::now(),
        //     upnp_lease_duration: time::Duration::from_secs(300), // TODO: Don't hard code this. Read from config file but also provide default value
        //     peers: vec![],
        // }
    }

    pub fn refresh_connection(&mut self) {
        match TcpListener::bind("0.0.0.0:0") {
            // Bind to empty port on all interfaces
            Ok(listener) => self.listener = Some(listener),
            Err(error) => {
                // self.status = ServerStatus::from(error.into());
                // self.status = ServerStatus::from(anyhow::anyhow!(error));
                // self.status = ServerStatus::TCPBindError;
                self.status = ServerStatus::TCPBindError(anyhow!(error));
                return;
            }
        }

        match get_local_ip() {
            Ok(ip) => self.local_ip = Some(ip),
            Err(error) => {
                // self.status = ServerStatus::from(anyhow::anyhow!(error));
                self.status = ServerStatus::LocalIPError(anyhow!(error));
                return;
            }
        };

        match get_public_ip() {
            Ok(ip) => self.public_ip = Some(ip),
            Err(error) => {
                // self.status = ServerStatus::from(error.into());
                self.status = ServerStatus::PublicIPError(anyhow!(error));
                return;
            }
        };

        // This unwrap will only be reached if the listener is ok above
        match self.listener.as_ref().unwrap().local_addr() {
            Ok(address) => self.internal_port = Some(address.port()),
            Err(error) => {
                // self.status = ServerStatus::from(error.into());
                self.status = ServerStatus::InternalPortError(anyhow!(error));
                return;
            }
        };

        match self.add_port_mapping() {
            Ok(port) => self.external_port = Some(port),
            Err(error) => {
                // self.status = ServerStatus::from(error.into());
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

    pub fn run(&mut self) {
        // Set up connection
        self.refresh_connection(); // Store result in status?
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
            .with_context(|| format!("No internal port."))?;
        let local_ip = self
            .local_ip
            .with_context(|| format!("No local IP address."))?;
        let local_ip = match local_ip {
            IpAddr::V4(ipv4) => Ok(ipv4),
            // IpAddr::V6(ipv6) => Err("Local IP is IPv6, but only IPv4 is supported.".into()),
            IpAddr::V6(ipv6) => Err(anyhow!(String::from(
                "Local IP is IPv6, but only IPv4 is supported."
            ))),
        }?;
        let local_address = SocketAddrV4::new(local_ip, internal_port);
        let gateway = igd::search_gateway(Default::default())
            .with_context(|| format!("Unable to find gateway device. Verify connection."))?;
        let external_port = gateway
            .add_any_port(
                igd::PortMappingProtocol::TCP,
                local_address,
                self.upnp_lease_duration
                    .as_secs()
                    .try_into()
                    .with_context(|| format!("UPnP lease duration should fit into u32"))?,
                "Bitgeon",
            )
            .with_context(|| format!("Unable to add port mapping via UPnP."))?;
        self.upnp_lease_clock = time::Instant::now();
        self.external_port = Some(external_port);
        Ok(external_port)
    }

    pub fn refresh_ips(&mut self) -> Result<()> {
        // Refresh connection info. Call this in case of errors trying to establish a connection, but also let the user call this.
        let local_ip =
            get_local_ip().with_context(|| format!("Unable to get local IP address."))?;
        self.local_ip = Some(local_ip);
        let public_ip =
            get_public_ip().with_context(|| format!("Unable to get public IP address."))?;
        self.public_ip = Some(public_ip);
        Ok(())
    }
}

pub struct Peer {
    tcp_stream: TcpStream,
}

pub fn get_public_ip() -> Result<IpAddr> {
    let gateway = igd::search_gateway(Default::default())
        .with_context(|| format!("Unable to find gateway. Verify your internet connection."))?;

    let ip = gateway
        .get_external_ip()
        .with_context(|| format!("Unable to get external IP address."))?;
    Ok(IpAddr::from(ip))
}

pub fn get_local_ip() -> Result<IpAddr> {
    // https://stackoverflow.com/a/166589/5780938

    let gateway = igd::search_gateway(Default::default())
        .with_context(|| format!("Unable to find gateway. Verify your internet connection."))?;

    // Bind to every available interface on an unused port
    let socket = UdpSocket::bind("0.0.0.0:0")
        .with_context(|| format!("Unable to bind UDP socket to \"0.0.0.0:0\"."))?;

    socket
        .connect(gateway.addr)
        .with_context(|| format!("Unable to connect to gateway."))?;

    let local_address = socket.local_addr()?;
    Ok(local_address.ip())
}
