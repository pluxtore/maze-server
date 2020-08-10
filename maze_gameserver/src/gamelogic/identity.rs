use super::clientkey::ClientKey;
use std::net::{SocketAddr,UdpSocket};
use std::hash::{Hash, Hasher};
use crate::aluminium::clientman::Message;
use super::*;

pub struct Identity {
    pub secret: ClientKey,
    pub socket: UdpSocket,
    pub peer_addr: SocketAddr,
    pub name: Vec<u8>,
    pub uid : [u8;4],
    pub handshake_payload : Vec<u8>,
}

impl Identity {
    pub fn new(handshake_msg: Message) -> Option<Self> {
        if handshake_msg.payload[0] != 0x4c || handshake_msg.payload.len()<42 {
            return None;
        }

        let mut name = handshake_msg.payload[10..].to_vec();
        let mut len = 0;

        for &byte in &name {
            if byte == 0x0 || byte == 0xa {
                break;
            } 
            len+=1;
        }
        name.truncate(len);

        Some(Self {
            secret: Logic::pkt_get_client_key(&handshake_msg.payload),
            socket: handshake_msg.socket,
            peer_addr: handshake_msg.peer_addr,
            name: name,
            uid : *b"0000",
            handshake_payload : handshake_msg.payload,
        })
    }

    pub fn get_key(&self) -> &ClientKey {
        &self.secret
    }
}
impl Eq for Identity {}
impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        self.get_key() == other.get_key()
    }
}

impl Hash for Identity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_key().hash(state);
    }
}