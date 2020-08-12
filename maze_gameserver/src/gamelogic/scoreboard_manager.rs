use std::net::TcpStream;
use std::io::prelude::*;
use std::time::Duration;
use super::score::Score;


const addr : &str = "127.0.0.1:42069";

pub struct ScoreboardManager {
    push_socket : TcpStream,
}

impl ScoreboardManager {
    pub fn new() -> Self {
        let s = match TcpStream::connect(addr) {
            Ok(e) => e,
            _ => {
                log::error!("could not connect to master server on {}", addr);
                panic!("could not connect to master server on {}", addr)
            }
        };
        match s.set_write_timeout(Some(Duration::from_millis(10))) {
            Ok(e) => e,
            _ => {
                log::error!("could not set write timeout on tcp socket");
            }
        }
        Self {
            push_socket : s,
        }
    }

    pub fn push_score(&mut self, score : &Score) {
        let mut msg = Vec::new();
        bincode::serialize_into(&mut msg, score).unwrap();
        match self.push_socket.write(msg.as_mut_slice()) {
            Ok(_) => (),
            _ => {
                log::error!("failed to push score update to master server")
            }
        }
    }
}