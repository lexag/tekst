use std::error::Error;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub struct NetworkWriterConfig {
    pub addr: SocketAddrV4,
}

impl Default for NetworkWriterConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 200), 10000),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Default)]
pub struct NetworkWriter {
    config: NetworkWriterConfig,
}

impl NetworkWriter {
    pub fn send_payload(&self, payload: &[u8]) -> Result<(), std::io::Error> {
        let mut stream = TcpStream::connect(self.config.addr)?;

        stream.write_all(payload)?;
        Ok(())
    }

    pub fn config_mut(&mut self) -> &mut NetworkWriterConfig {
        &mut self.config
    }
}
