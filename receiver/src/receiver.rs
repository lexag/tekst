use local_ip_address::local_ip;
use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    str::FromStr,
};
use tekst_common::protocol::Message;

pub struct Receiver {
    pub listener: TcpListener,
}

impl Receiver {
    pub fn new() -> Self {
        let listener = TcpListener::bind(SocketAddrV4::new(
            Ipv4Addr::from_str(&local_ip().unwrap().to_string()).unwrap(),
            10000,
        ))
        .unwrap();
        listener.set_nonblocking(true);
        Self { listener }
    }

    pub fn rcv(&self) -> Option<Message> {
        let Ok((mut stream, addr)) = self.listener.accept() else {
            //println!("faulty TCP accept (or nothing received)");
            return None;
        };

        let mut buf = String::new();
        let Ok(bytes_read) = stream.read_to_string(&mut buf) else {
            println!("faulty TCP read");
            return None;
        };

        match serde_json::from_str(&buf) {
            Ok(val) => val,
            Err(e) => {
                println!("faulty parse: {}\n {}", e, buf);
                None
            }
        }
    }
}
