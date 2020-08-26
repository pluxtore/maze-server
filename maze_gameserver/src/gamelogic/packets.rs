use std::mem;
use super::*;
use super::map::{LocationType,Flag};
use super::types::*;
use byteorder::{LittleEndian, WriteBytesExt};
use std::convert::TryInto;
use super::score::Score;

/* ######## packet types  ######## */
#[derive(Clone)]
pub struct PosUpdate {
    uid : [u8;4],
    timestamp : u32,
    pos_x : i32,
    pos_y : i32,
    pos_z : i32,
    euler_x : i32,
    euler_y : i32,
    euler_z : i32,
    add_ren_info : [u8;5],
}

#[derive(Clone)]
pub struct Emoji {
    uid : [u8;4],
    emoji : u8,
}

impl Logic {
    pub fn parse_login_pkt(&mut self, payload : &mut [u8]) -> PacketParseResult<Self>{
        if payload.len() < 42 {
            return PacketParseResult::HardError;
        }
        /* login response */
        let mut msg = Vec::new();
        if self.npacket == 0 {
            self.id.uid = rand::random::<[u8;4]>();
            while uid_dict.lock().unwrap().get(&self.id.uid).is_some() {
                self.id.uid = rand::random::<[u8;4]>();
            }
            uid_dict.lock().unwrap().insert(self.id.uid, (self.id.name.clone(), self.cs.unlocks, self.cs.color));
        } 
        msg.append(&mut self.id.uid.to_vec());
        msg.push(self.cs.unlocks.get_raw());
        msg.push(self.cs.color.get_raw());
        msg.push(*client_version);
        self.send(0x4c, &mut msg);
        msg.clear();

        self.client_time = 0;
        PacketParseResult::Ignore
    }

    pub fn parse_position_pkt(&mut self, payload : &mut [u8]) -> PacketParseResult<PosUpdate> {
        if payload.len() != 46 {
            return PacketParseResult::HardError
        }

        if timer.elapsed().as_secs_f32()-self.last_posupdate<0.05 { // cap position updates at 20Hz
            return PacketParseResult::Ignore
        }

        let pkt_uid = self.get_uid();
        let pkt_timestamp = unsafe { mem::transmute::<[u8; 4], u32>(payload[9..13].try_into().unwrap()) };
        let pkt_pos_x = unsafe { mem::transmute::<[u8; 4], i32>(payload[17..21].try_into().unwrap()) };
        let pkt_pos_y = unsafe { mem::transmute::<[u8; 4], i32>(payload[21..25].try_into().unwrap()) };
        let pkt_pos_z = unsafe { mem::transmute::<[u8; 4], i32>(payload[25..29].try_into().unwrap()) };
        let pkt_euler_x = unsafe { mem::transmute::<[u8; 4], i32>(payload[29..33].try_into().unwrap()) };
        let pkt_euler_y = unsafe { mem::transmute::<[u8; 4], i32>(payload[33..37].try_into().unwrap()) };
        let pkt_euler_z = unsafe { mem::transmute::<[u8; 4], i32>(payload[37..41].try_into().unwrap()) };
        let pkt_add_ren_info = payload[41..46].try_into().unwrap();


        if pkt_timestamp < self.client_time as u32 {
            return PacketParseResult::Ignore;
        }

        /* "anti speed hack" */ /* originally, i thought you had to tamper the timestamp, but apperently you should not go more than 10000units per update */
        {
            let (oldx,_,oldz) =  self.cs.getpos();
            let (relx,relz) : (i64,i64) = ( ( oldx-pkt_pos_x ).abs() as i64,  ( oldz-pkt_pos_z ).abs() as i64);
            let travelled_distance = ( ( ( i64::pow(relx,2)  + i64::pow(relz,2) ) )  as f64 ).sqrt();
            if travelled_distance > *max_distance_per_packet {
                self.send_teleport(Place::Custom);
                return PacketParseResult::Ignore;
            }
        }


        match mazemap.eval_position(pkt_pos_x,pkt_pos_z) {
            LocationType::Allowed => {
                self.cs.setpos(pkt_pos_x, pkt_pos_y, pkt_pos_z);
                self.client_time = pkt_timestamp as u64;
            },
            LocationType::NotAllowed => {
                self.send_teleport(Place::Custom);
                return PacketParseResult::Ignore;
            },
            LocationType::OnPlace(place) => {
                match place {
                    Place::RaceStart => {
                        if !self.cs.unlocks.get(1) {
                            self.cs.unlocks.set(1);
                            if self.cs.unlocks.get_total()>=4 {
                                self.cs.color.make_white();
                            }
                            self.send_unlocks();
                        }

                    },
                    Place::Hangar => {
                        if !self.cs.unlocks.get(3) {
                            self.cs.unlocks.set(3);
                            if self.cs.unlocks.get_total()>=4 {
                                self.cs.color.make_white();
                            }
                            self.send_unlocks();
                        }
                    },
                    Place::Lava => {
                        if !self.cs.unlocks.get(2) {
                            self.cs.unlocks.set(2);
                            if self.cs.unlocks.get_total()>=4 {
                                self.cs.color.make_white();
                            }
                            self.send_unlocks();
                        }
                    },
                    Place::Tower => {
                        if !self.cs.unlocks.get(0) {
                            self.cs.unlocks.set(0);
                            if self.cs.unlocks.get_total()>=4 {
                                self.cs.color.make_white();
                            }
                            self.send_unlocks();

                        }
                    },
                    _ => (),
                }
                self.cs.setpos(pkt_pos_x, pkt_pos_y, pkt_pos_z);
                self.client_time = pkt_timestamp as u64;
            }
            LocationType::TeleportAway(place) => {
                let is_ok = match place {
                    Place::RaceStart => {self.cs.unlocks.get(1)},
                    Place::Hangar => {self.cs.unlocks.get(3)},
                    Place::Lava => {self.cs.unlocks.get(2)},
                    Place::Tower => {self.cs.unlocks.get(0)},
                    _ => false,
                };
                if is_ok {
                    self.send_teleport(place);
                } else {
                    self.send_text("Teleporter locked. Area not discovered yet.");
                    self.cs.setpos(pkt_pos_x, pkt_pos_y, pkt_pos_z);
                }
                self.client_time = pkt_timestamp as u64;
            },
            LocationType::DieZone => {
                if pkt_pos_y<5000 && pkt_pos_y>-5000 {
                    self.send(0x44, &mut b"\x20\x00".to_vec()); // death packet
                    self.cs.set_alive(false);
                }
                self.cs.setpos(pkt_pos_x, pkt_pos_y, pkt_pos_z);
                self.client_time = pkt_timestamp as u64;
            }
            LocationType::FlagArea(flag) => {
                match flag {
                    Flag::Lava => {
                        self.send_flag("{U_haz_been_banned_by_B4TTL3Y3}", true);
                        if !self.cs.unlocks.get(4) {
                            self.cs.unlocks.set(4);
                            if self.cs.unlocks.get_total()>=4 {
                                self.cs.color.make_white();
                            }
                            self.send_unlocks();

                        }
                    }
                    Flag::Tower => {
                        if pkt_pos_y>471998 {
                            self.send_flag("{D1d_you_so1v3_th3_m4z3?}", true);
                            if !self.cs.unlocks.get(5) {
                                self.cs.unlocks.set(5);
                                if self.cs.unlocks.get_total()>=4 {
                                    self.cs.color.make_white();
                                }
                                self.send_unlocks();
                            }
                        }
                    }
                }
                self.cs.setpos(pkt_pos_x, pkt_pos_y, pkt_pos_z);
                self.client_time = pkt_timestamp as u64;
            }
            LocationType::Checkpoint(checkpoint) => {
                if self.race_manager.eval(checkpoint) {
                    self.send_checkpoint(checkpoint);

                    if checkpoint == 12 {
                        let score = self.race_manager.get_total_time();

                        if self.cs.get_highscore() > score {
                            log::debug!("updating hs from {} to {}", self.cs.get_highscore(), score);
                            push_score(&Score::new(self.id.get_key().clone(), self.id.name.clone(),score));
                            self.cs.set_highscore(score);
                        }

                        if score < 5.0 {
                            self.send_flag("{fffffresh}",true);
                            self.race_manager.reset();
                        }

                        self.send_flag("{tHosE_aRe_jUsT_gAmiNg_sKiLls}", true);
                        if !self.cs.unlocks.get(6) {
                            self.cs.unlocks.set(6);
                            if self.cs.unlocks.get_total()>=4 {
                                self.cs.color.make_white();
                            }
                            self.send_unlocks();
                        }
                    }

                }
                self.cs.setpos(pkt_pos_x, pkt_pos_y, pkt_pos_z);
                self.client_time = pkt_timestamp as u64;
            }
        };

        self.last_posupdate = timer.elapsed().as_secs_f32();
        PacketParseResult::OkDispatch(PosUpdate{
            uid : pkt_uid,
            timestamp : pkt_timestamp,
            pos_x : pkt_pos_x,
            pos_y : pkt_pos_y,
            pos_z : pkt_pos_z,
            euler_x : pkt_euler_x,
            euler_y : pkt_euler_y,
            euler_z : pkt_euler_z,
            add_ren_info : pkt_add_ren_info,
        })
    }


    pub fn parse_heartbeat_pkt(&mut self, payload : &mut [u8]) -> PacketParseResult<()>{
        if payload.len() != 18 {
            return PacketParseResult::HardError;
        }
        let pkt_timestamp = unsafe { mem::transmute::<[u8; 8], u64>(payload[10..18].try_into().unwrap()) };

        if pkt_timestamp < self.client_time {
            return PacketParseResult::Ignore;
        }


        let mut msg = payload[10..18].to_vec();
        let mut bs = [0u8; mem::size_of::<i64>()];
        match bs
            .as_mut()
            .write_u64::<LittleEndian>( ( timer.elapsed().as_nanos() / 10000000 ) as u64)
        {
            _ => (),
        };
        msg.append(&mut bs.to_vec());
        //msg.append(&mut );
        self.send(0x3c, &mut msg);
        PacketParseResult::Ignore
    }


    pub fn parse_emoji_pkt(&mut self, payload : &mut [u8]) -> PacketParseResult<Emoji>{
        if payload.len() != 10 {
            return PacketParseResult::HardError;
        }

        if timer.elapsed().as_secs_f32()-self.last_emoji<3.0 {
            return PacketParseResult::Ignore;
        }

        let pkt_emoji = payload[9];
        let pkt_uid = self.id.uid;
        
        let mut timestamp = [0u8; mem::size_of::<u32>()];
        match timestamp.as_mut().write_u32::<LittleEndian>(timer.elapsed().as_secs_f32() as u32) {
            _ => (),
        };

        let mut msg = Vec::new();
        msg.append(&mut pkt_uid.to_vec());
        msg.append(&mut timestamp.to_vec());
        msg.push(pkt_emoji);
        self.send(0x45, &mut msg);
        if pkt_emoji == 0x0d {
            self.send_flag("{Th3_BND_w4ntz2know_your_loc4tion}", true);
            if !self.cs.unlocks.get(7) {
                self.cs.unlocks.set(7);
                if self.cs.unlocks.get_total()>=4 {
                    self.cs.color.make_white();
                }
                self.send_unlocks();
            }
        }
        self.last_emoji = timer.elapsed().as_secs_f32();
        PacketParseResult::OkDispatch(
            Emoji {
                uid : pkt_uid,
                emoji : pkt_emoji,
            }
        )
    }


    pub fn parse_info_pkt(&mut self, payload : &mut [u8]) -> PacketParseResult<()> {
        if payload.len() != 13 {
            return PacketParseResult::HardError;
        }

        let mut playerinfo = match uid_dict.lock().unwrap().get( &mut payload[9..13]) {
            Some(e) => e.clone(),
            _ => {log::warn!("client [{:x?}] sent bad info request (invalid uid : {:x?})", &self.id.get_key(), &payload[9..13]);return PacketParseResult::Ignore},
        };

        let mut msg = Vec::new();
        msg.append(&mut payload[9..13].to_vec()); // uid
        msg.push(playerinfo.1.get_raw()); // unlocks
        msg.push(playerinfo.2.get_raw()); // color
        msg.push(playerinfo.0.len() as u8);
        msg.append(&mut playerinfo.0);
        self.send(0x49, &mut msg); 
        PacketParseResult::Ignore
    }


    pub fn send_text(&mut self, text : &str) {
        if timer.elapsed().as_secs_f32()-self.last_text>7.0 {
            self.send(0x20, &mut text.as_bytes().to_vec());
            self.last_text = timer.elapsed().as_secs_f32();
        }
    }

    pub fn send_flag(&mut self, flag : &str, cooldown : bool) {
        if timer.elapsed().as_secs_f32()-self.last_flag>2.0 || !cooldown {
            let mut msg = Vec::new();
            msg.append(&mut b"SCG".to_vec());
            msg.append(&mut flag.as_bytes().to_vec());
            self.send(0x43, &mut msg);
            self.last_flag = timer.elapsed().as_secs_f32();
        }
    }

    pub fn send_checkpoint(&mut self, checkpoint : u8) {
        self.send(0x52, &mut [checkpoint].to_vec());
    }

    pub fn send_unlocks(&mut self) {
        let mut msg = Vec::new();
        msg.push(self.cs.unlocks.get_raw()); // new emoji
        msg.push(self.cs.color.get_raw()); // rabbitcolor
        self.send(0x55, &mut msg);
        self.has_new_unlock = true;
    }

    pub fn send_teleport(&mut self, place : Place) {

        let (x,y,z) = match place {
            Place::Hangar => *hangar_spawn,
            Place::Lava => *lava_spawn,
            Place::RaceStart => *race_spawn,
            Place::Tower => *tower_spawn,
            Place::Mid => *mid_spawn,
            Place::RandomMid => random_mid_spawn(),
            Place::Custom => self.cs.getpos()
        };

        self.cs.setpos(x, y, z);
    
        let mut msg = Vec::new();
        msg.push(1);
        let mut bs = [0u8; mem::size_of::<i32>()];
        match bs.as_mut().write_i32::<LittleEndian>(x) {
            _ => (),
        };
        msg.append(&mut bs.to_vec());
        match bs.as_mut().write_i32::<LittleEndian>(y) {
            _ => (),
        };
        msg.append(&mut bs.to_vec());
        match bs.as_mut().write_i32::<LittleEndian>(z) {
            _ => (),
        };
        msg.append(&mut bs.to_vec());
        self.send(0x54, &mut msg);
    }

    pub fn kick_client(&mut self) {
        self.send(0x58, &mut Vec::new());
    }
}

impl Dispatch for PosUpdate {
    fn dispatch(&self, _logic : &Logic) -> Option<(Vec<u8>,DispatchType)> {        
        let mut msg = Vec::new();
        let mut bs = [0u8; mem::size_of::<i32>()];

        msg.append(&mut self.uid.to_vec());
        


        match bs.as_mut().write_u32::<LittleEndian>(self.timestamp) {
            Ok(_e) => {msg.append(&mut bs.to_vec())},
            _ => return None
        };

        msg.append(&mut b"\x00\x00\x00\x00".to_vec());

        match bs.as_mut().write_i32::<LittleEndian>(self.pos_x) {
            Ok(_e) => {msg.append(&mut bs.to_vec())},
            _ => return None,
        };

        match bs.as_mut().write_i32::<LittleEndian>(self.pos_y) {
            Ok(_e) => {msg.append(&mut bs.to_vec())},
            _ => return None,
        };

        match bs.as_mut().write_i32::<LittleEndian>(self.pos_z) {
            Ok(_e) => {msg.append(&mut bs.to_vec())},
            _ => return None,
        };

        match bs.as_mut().write_i32::<LittleEndian>(self.euler_x) {
            Ok(_e) => {msg.append(&mut bs.to_vec())},
            _ => return None,
        };

        match bs.as_mut().write_i32::<LittleEndian>(self.euler_y) {
            Ok(_e) => {msg.append(&mut bs.to_vec())},
            _ => return None,
        };

        match bs.as_mut().write_i32::<LittleEndian>(self.euler_z) {
            Ok(_e) => {msg.append(&mut bs.to_vec())},
            _ => return None,
        };

        msg.append(&mut self.add_ren_info.to_vec());


        Some((msg,DispatchType::PosUpdate))
    }
}

impl Dispatch for Emoji {
    fn dispatch(&self, _logic : &Logic) -> Option<(Vec<u8>,DispatchType)> {
        let mut msg = Vec::new();

        msg.append(&mut self.uid.to_vec());
        let mut timestamp = [0u8; mem::size_of::<u32>()];
        match timestamp.as_mut().write_u32::<LittleEndian>(timer.elapsed().as_secs_f32() as u32) {
            _ => (),
        };
        msg.append(&mut timestamp.to_vec());
        msg.push(self.emoji);
        Some((msg,DispatchType::Emoji))
    }
}
