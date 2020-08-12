pub mod clientkey;
pub mod types;
mod characterstate;
mod identity;
mod map;
mod packets;
mod rabbitcolor;
mod unlocks;
mod antispam;
mod raceman;
mod scoreboard_manager;
mod score;

use std::sync::{Arc,Mutex};
use std::collections::{HashMap};
use std::time::Instant;

use std::convert::TryInto;
use characterstate::CharacterState;
use identity::Identity;
use unlocks::Unlocks;
use rabbitcolor::RabbitColor;
use map::{Map,Place};
use crate::aluminium::clientman::{Message,ParseFeedback};
use clientkey::ClientKey;
use types::*;
use antispam::AntiSpam;
use raceman::RaceManager;
use scoreboard_manager::ScoreboardManager;

#[macro_use]

lazy_static::lazy_static! {
    pub static ref client_version: u8 = 2;
    pub static ref client_thread_timeout : u64 = 30000; // you can reconnect until this time is up
    pub static ref timer : std::time::Instant = Instant::now();
    pub static ref uid_dict : Mutex<HashMap<[u8;4], (Vec<u8>, Unlocks,RabbitColor)>> = Mutex::new(HashMap::new());
    pub static ref mazemap : Map = Map::new("maze.map");
    pub static ref sboard : Mutex<ScoreboardManager> = Mutex::new(ScoreboardManager::new());
    pub static ref packets_per_second_limit : u16 = 35;
    pub static ref max_distance_per_packet : f64 = 100000.0;
    pub static ref mid_spawn : (i32,i32,i32 ) = (2658 * 1000, 10 *10000, 2393 * 1000);
    pub static ref tower_spawn : (i32,i32,i32) = (380365, 3 * 10000, 4019494);
    pub static ref race_spawn : (i32,i32,i32) = (2124889, 3 * 10000, 1995128);
    pub static ref lava_spawn : (i32,i32,i32) = (4052942, 3 * 10000, 4514700);
    pub static ref hangar_spawn : (i32,i32,i32) = (1812125, 3 * 10000, 645054);
}


pub fn random_mid_spawn() -> (i32,i32,i32) {
    let mut spawn = *mid_spawn;
    spawn.0 += rand::random::<i32>() % 80000;
    spawn.2 += rand::random::<i32>() % 80000;
    spawn
}


pub struct Logic {
    pub cs : CharacterState,
    id : Identity,
    npacket: u128,
    client_time: u64,

    /* cooldown related */ 
    last_posupdate : f32,
    last_emoji : f32,
    last_text : f32,
    last_flag : f32,

    has_new_unlock : bool,
    anti_spam : AntiSpam,
    race_manager : RaceManager,
}

impl Logic {
    /* Logic instance should be inside the client thread*/

    /* SECTION 1 - PUBLIC INTERFACE */

    /* new client logic */
    pub fn new(message : Message) -> Option<Self> {
        
        let identity = match Identity::new(message.clone()) {
            Some(e) => e,
            None => {
                return None;
            }
        };

        let state = match CharacterState::try_activate(identity.get_key()) {
            Some(mut e) => {
                log::info!(
                    "existing client {:x?} at {} loaded and entering runmethod",
                    identity.get_key(),
                    identity.peer_addr
                );
                e.update_name(&identity.name);
                e
            }
            None => {
                log::info!(
                    "new client with idnum {:x?} at {} created and entering runmethod",
                    identity.get_key(),
                    identity.peer_addr
                );
                CharacterState::new(&identity.name)
            }

        };

        let mut preself = Self {
            cs : state,
            id : identity,
            npacket: 0,
            client_time: 0,
            last_posupdate : timer.elapsed().as_secs_f32(),
            last_emoji : timer.elapsed().as_secs_f32(),
            last_text : timer.elapsed().as_secs_f32(),
            last_flag : timer.elapsed().as_secs_f32(),
            has_new_unlock : false,
            anti_spam : AntiSpam::new(),
            race_manager : RaceManager::new(),
        };    

        preself.inbound_parse(message);
        Some(preself)
    }

    /* encryption and decryption related */ 
    pub fn decode_pkt(buf: &[u8]) -> Vec<u8> {
        let mut firstrandom: u8 = buf[0];
        let secondrandom: u8 = buf[1];
        let mut decoded = Vec::new();
    
        for i in 0..buf.len() - 2 {
            decoded.push(buf[i + 2] ^ firstrandom);
            let v21: u64 = (firstrandom as u64 + secondrandom as u64) as u64;
            firstrandom = (v21 + ((2155905153 * (v21)) >> 39)) as u8;
        }
        decoded
    }
    
    pub fn encode_pkt(buf: Vec<u8>) -> Vec<u8> {
        let mut firstrandom = rand::random::<u8>();
        let secondrandom = rand::random::<u8>();
        let mut encoded = Vec::new();
        encoded.push(firstrandom);
        encoded.push(secondrandom);
    
        for i in 0..buf.len() {
            encoded.push(buf[i] ^ firstrandom);
            let v21: u64 = (firstrandom as u64 + secondrandom as u64) as u64;
            firstrandom = (v21 + ((2155905153 * (v21)) >> 39)) as u8;
        }
        encoded
    }

    /*once decrypted, how to route packet ? */
    pub fn pkt_get_client_key(payload: &Vec<u8>) -> ClientKey {
        if payload[0] == 0x3c {
            return ClientKey::new(payload.as_slice()[2..10].try_into().unwrap());
        }
        ClientKey::new(payload.as_slice()[1..9].try_into().unwrap())
    }


    /* every new packet for this client is parsed here */
    pub fn inbound_parse(
        &mut self,
        mut message : Message,
    ) -> ParseFeedback<Arc<Mutex<dyn Dispatch + Send>>> {

        if !self.anti_spam.tick_and_is_ok() {
            self.kick_client();
            return ParseFeedback::ForceTermination;
        }


        let operation = message.payload[0];

        {
            if !self.cs.is_alive() && (operation==0x45 || operation==0x50) {
                self.send_text("you died, didn't you?");
                self.cs.set_alive(true);
                self.send_teleport(Place::RandomMid);
            }

            if !self.cs.is_alive() && operation==0x4c {
                self.send_text("you died, didn't you?");
                self.cs.set_alive(true);
            }

            if self.has_new_unlock { // update entry if rabbit went white
                self.has_new_unlock = false; 
                let mut dict = uid_dict.lock().unwrap();
                match dict.remove(&self.id.uid) {
                    _ => (),
                }
                dict.insert(self.id.uid, (self.id.name.clone(),self.cs.unlocks,self.cs.color));
            }
        }

        match operation {
            0x4c => { // login packet
                self.id.peer_addr = message.peer_addr;
                match self.parse_login_pkt(message.payload.as_mut_slice()) {
                    PacketParseResult::HardError => {self.kick_client();return ParseFeedback::ForceTermination},
                    _ => (),
                }
                self.send_teleport(Place::RandomMid);
            }
            0x3c => { // heartbeat packet
                match self.parse_heartbeat_pkt(message.payload.as_mut_slice()) {
                    PacketParseResult::HardError => {self.kick_client();return ParseFeedback::ForceTermination},
                    _ => (),
                }
            }
            0x50 => { // position packet
                match self.parse_position_pkt(message.payload.as_mut_slice()) {
                    PacketParseResult::OkDispatch(update) => return ParseFeedback::NewEvent(Arc::new(Mutex::new(update))),
                    PacketParseResult::HardError => {self.kick_client();return ParseFeedback::ForceTermination},
                    _ => (),
                }
            }
            0x45 => { // emoji packet
                match self.parse_emoji_pkt(message.payload.as_mut_slice()) {
                    PacketParseResult::OkDispatch(update) => return ParseFeedback::NewEvent(Arc::new(Mutex::new(update))),
                    PacketParseResult::HardError => {self.kick_client();return ParseFeedback::ForceTermination},
                    _ => (),
                }
            }
            0x49 => { // info packet
                match self.parse_info_pkt(message.payload.as_mut_slice()) {
                    PacketParseResult::HardError => {self.kick_client();return ParseFeedback::ForceTermination}
                    _ => (),
                }
            }
            _ => {
                self.kick_client();
                return ParseFeedback::ForceTermination
            }
        };
        ParseFeedback::PullNewEvents
    }

    pub fn outbound_create(&mut self, events: Option<Vec<Arc<Mutex<dyn Dispatch + Send>>>>) {
        let mut msg : Vec<u8>= Vec::new();
        
        match events {
            Some(e) => {
                match e.is_empty() {
                    false => {
                        for event in e {
                            let update = event.lock().unwrap().dispatch(self);

                            match update {
                                Some(mut e) => {
                                    match e.1 {
                                        DispatchType::PosUpdate => {
                                            if msg.len() > 256 {
                                                self.send(0x50, &mut msg);
                                            } else {
                                                if msg.len()!=0 {
                                                    msg.push(0x50);
                                                }
                                                msg.append(&mut e.0);
                                            }
                                        }
                                        DispatchType::Emoji => {
                                            if msg.len() != 0 {
                                                self.send(0x50, &mut msg);
                                            }
                                            self.send(0x45, &mut e.0);

                                        }
                                    }
                                },
                                None => {}
                            };
                        }
                        if msg.len() != 0 {
                            self.send(0x50, &mut msg);
                        }
                    }   
                    _ => (),
                }
            }
            None => {()}
        };
    }


    pub fn get_state(&mut self) -> &mut CharacterState {
        &mut self.cs
    }

    pub fn get_identitiy(&self) -> &Identity {
        &self.id
    }

    pub fn shutdown(&mut self) {
        uid_dict.lock().unwrap().remove(&self.id.uid);
        self.cs.shutdown(self.id.get_key());
    }

    pub fn get_uid(&self) -> [u8;4] {
        self.id.uid
    }


    /*SECTION 2 - PRIVATE INTERFACE */ 

    fn send(&mut self, operation: u8, msg: &mut Vec<u8>) -> bool {
        let mut to_encode = Vec::new();
        to_encode.push(operation);
        if operation == 0x3c {
            to_encode.push(0x33);
        }
        to_encode.append(msg);
        self.npacket += 1;

        match self.id
            .socket
            .send_to(Logic::encode_pkt(to_encode).as_slice(), self.id.peer_addr)
        {
            Ok(_e) => return true,
            _ => {
                log::warn!("could not send packet back to {}", self.id.peer_addr);
                return false;
            }
        }
    }
}
