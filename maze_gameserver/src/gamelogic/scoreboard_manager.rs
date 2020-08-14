use std::net::TcpStream;
use std::io::prelude::*;
use std::time::Duration;
use super::score::Score;


const addr : &str = "127.0.0.1:42069";


pub fn push_score(score : &Score) {

    let mut s = match TcpStream::connect(addr) {
        Ok(e) => e,
        _ => {
            log::error!("could not connect to master server on {}", addr);
            panic!("could not connect to master server on {}", addr)
        }
    };

    match s.set_write_timeout(Some(Duration::from_millis(50))) {
        Ok(e) => e,
        _ => {
            log::error!("could not set write timeout on tcp socket");
        }
    }

    let mut msg = Vec::new();
    bincode::serialize_into(&mut msg, score).unwrap();

    match s.write(msg.as_mut_slice()) {
        Ok(_) => (),
        _ => {
            log::error!("failed to push score update to master server")
        }
    }
}
