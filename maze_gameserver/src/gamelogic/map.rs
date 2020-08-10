use std::fs::File;
use std::io::prelude::*;

pub struct Map {
    in_mem_map : Vec<Vec<u8>>,
}

impl Map {
    pub fn new(fname : &str) -> Self {
        let mut file = match File::open(fname) {
            Ok(e) => e,
            _ => {log::error!("could not open map file");panic!("could not open map file");}
        };

        let mut tmp = [0;1068];
        let mut finl = Vec::new();

        for _ in 0..1068 {
            match file.read(&mut tmp) {
                Ok(_) => (),
                _ => {log::error!("could not read map file");panic!("could not read map file")}
            }
            finl.push(tmp.to_vec());
        }

        Self {
            in_mem_map : finl,
        }
    }

    pub fn eval_position(&self, mut x: i32, mut z : i32) -> LocationType {
        x-=41004;
        z-=24735;
        
        let mut floatx = x as f32;
        let mut floatz = z as f32;

        floatx /= 4497.191011236;
        floatz /= 4497.191011236;
        
        let (x,z) = (floatx as i32, floatz as i32);

        if (0<= x && x<1067) && (0<= z && z<1067) {
            match self.in_mem_map.get( z as usize ).unwrap().get( x as usize ).unwrap() {
                0x82 => return LocationType::OnPlace(Place::Lava),
                0x83 => return LocationType::OnPlace(Place::Tower),
                0x84 => return LocationType::OnPlace(Place::RaceStart),
                0x85 => return LocationType::OnPlace(Place::Hangar),

                0x00 => return LocationType::Allowed,
                0xff => return LocationType::NotAllowed,
                0x40 => return LocationType::FlagArea(Flag::Lava),
                0x41 => return LocationType::FlagArea(Flag::Tower),
                0x7c => return LocationType::DieZone,
                
                0xd0 => return LocationType::TeleportAway(Place::Hangar),
                0xd1 => return LocationType::TeleportAway(Place::Tower),
                0xd2 => return LocationType::TeleportAway(Place::RaceStart),
                0xd3 => return LocationType::TeleportAway(Place::Lava),
                0x6c => return LocationType::Checkpoint(0),
                0x60 => return LocationType::Checkpoint(1),
                0x61 => return LocationType::Checkpoint(2),
                0x62 => return LocationType::Checkpoint(3),
                0x63 => return LocationType::Checkpoint(4),
                0x64 => return LocationType::Checkpoint(5),
                0x65 => return LocationType::Checkpoint(6),
                0x66 => return LocationType::Checkpoint(7),
                0x67 => return LocationType::Checkpoint(8),
                0x68 => return LocationType::Checkpoint(9),
                0x69 => return LocationType::Checkpoint(10),
                0x6a => return LocationType::Checkpoint(11),
                0x6b => return LocationType::Checkpoint(12),

                _ => return LocationType::NotAllowed,
            };
        }

        LocationType::NotAllowed
    }
}



pub enum Place {
    RaceStart,
    Hangar,
    Tower,
    Lava,
    Mid,
    RandomMid,
    Custom,
}

pub enum Flag {
    Tower,
    Lava
}




pub enum LocationType {
    Allowed,
    NotAllowed,
    DieZone,
    FlagArea(Flag),
    TeleportAway(Place),
    OnPlace(Place),
    Checkpoint(u8),
}
