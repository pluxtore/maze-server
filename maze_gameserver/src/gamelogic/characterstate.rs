use super::unlocks::Unlocks;
use super::rabbitcolor::RabbitColor;
use std::fs::File;
use serde::{Serialize, Deserialize};
use super::clientkey::ClientKey;
use std::io::prelude::*;
use std::ops::{Add};

#[derive(Serialize, Deserialize)]
pub struct CharacterState {
    name: Vec<u8>,
    position: (i32, i32, i32),
    is_alive: bool,
    is_new: bool,
    pub unlocks: Unlocks,
    pub color : RabbitColor,
    highscore : f32,
}

impl CharacterState {
    pub fn new(name: &Vec<u8>) -> Self {
        Self {
            position: super::random_mid_spawn(),
            name: name.clone(),
            is_alive: true,
            is_new: true,
            unlocks: Unlocks::new(),
            color : RabbitColor::new(),
            highscore : 100000000000000.0,
        }
    }
    pub fn setpos(&mut self, x: i32, y: i32, z: i32) {
        self.position.0 = x;
        self.position.1 = y;
        self.position.2 = z;
    }
    pub fn is_alive(&self) -> bool {
        self.is_alive
    }

    pub fn set_alive(&mut self,value : bool) {
        self.is_alive = value;
    }

    pub fn update_name(&mut self, newname: &Vec<u8>) {
        self.name = newname.clone();
    }

    pub fn set_highscore(&mut self, hs : f32) {
        self.highscore = hs;
    }

    pub fn get_highscore(&self) -> f32 {
        self.highscore
    }

    pub fn shutdown(&mut self, key : &ClientKey) {
        self.is_new = false;
        let mut f = match File::create("clients/".to_string().add(hex::encode(key.get_raw_bytes()).as_str())) {
            Ok(e) => e,
            Err(_e) => {
                log::error!("no such directory \"clients\"");
                return;
            }
        };
        bincode::serialize_into(&mut f, self).unwrap();
    }

    pub fn try_activate(key: &ClientKey) -> Option<Self> {
        let mut f = match File::open("clients/".to_string().add(hex::encode(key.get_raw_bytes()).as_str())) {
            Ok(e) => e,
            Err(_e) => return None,
        };

        let mut readbuf = Vec::new();

        match f.read_to_end(&mut readbuf) {
            Ok(e) => e,
            Err(_e) => {
                println!("could not read from file");
                return None;
            }
        };

        Some(bincode::deserialize(&readbuf[..]).unwrap())
    }

    pub fn getpos(&self) -> (i32, i32, i32) {
        self.position
    }
}