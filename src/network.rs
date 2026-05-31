use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct ConnectionSettings {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            ip: Ipv4Addr::new(192, 168, 1, 200),
            port: 10000,
        }
    }
}

pub fn send_payload(payload: Vec<u8>, settings: ConnectionSettings) -> Option<()> {
    let mut stream = TcpStream::connect(SocketAddrV4::new(settings.ip, settings.port)).ok()?;

    stream.write(&payload);
    None
}
