#![feature(proc_macro_hygiene, decl_macro)]
#![allow(non_upper_case_globals)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

mod clientkey;
mod score;

use std::sync::Mutex;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::prelude::*;
use clientkey::ClientKey;
use score::Score;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::convert::TryInto;

lazy_static! {
    pub static ref board : Mutex<String> = Mutex::new(String::new());
    pub static ref allscores : Mutex<HashMap<ClientKey, f32>> = Mutex::new(HashMap::new());
}

#[get("/api/<req>")]
fn api(req: String) -> String {
    match req.as_str() {
        "health" => String::from("yes"),
        "min_port" => String::from("1337"),
        "max_port" => String::from("1338"),
        "user_queue" => String::from("0"),
        "rate_limit" => String::from("20"),
        "hostname" => String::from("maze.pluxtore.de"),
        "highscore" => board.lock().unwrap().clone(),
        "welcome" => String::from("you are not using the official server"),
        _ => String::from("invalid req lul :/"),
    }
}


#[get("/api/highscore/<req>")]
fn highscore(req : String) -> String {
    let key = match hex::decode(req.clone()) {
        Ok(e) => e,
        _ => return "".to_string()
    };
    if key.len() != 8 {
        return "".to_string();
    }
    let ckey = ClientKey::new(key.as_slice().try_into().unwrap());


    match allscores.lock().unwrap().get(&ckey) {
        Some(e) => {
            let mut hs = e.to_string();
            match hs.split_off(3) {
                _ => (),
            }
            return hs;
        },
        None => {
            return "".to_string();
        },
    };
}


fn main() {
    std::thread::spawn(move || {
        Handler::new().handler();        
    });
    rocket::ignite().mount("/", routes![api,highscore]).launch();
}

struct Handler {
    scorelist : HashMap<ClientKey, (f32, String)>,
    lowest_hs : f32
}

impl Handler {
    fn new () -> Self {
        Self {
            scorelist : HashMap::new(),
            lowest_hs : 50.0,
        }
    }

    fn handler(&mut self) {
        let listener = match TcpListener::bind("127.0.0.1:42069") {
            Ok(e) => e,
            _ => {
                panic!("could not bind to socket");
            }
        };

        loop {
            for stream in listener.incoming() {
                match stream {
                    Ok(e) => {
                        self.process(e);
                    }
                    _ => {
                    }
                }
            }
        }
    }

    fn process(&mut self,mut stream : TcpStream) {
        let mut buf : [u8;500] = [0;500];
        let len  = match stream.read(&mut buf) {
            Ok(e) => e,
            _ => return,
        };

        match stream.shutdown(Shutdown::Both) {
            _ => (),
        }


        let score : Score = bincode::deserialize(&buf[..len]).unwrap();

        if !allscores.lock().unwrap().contains_key(&score.key) {
            allscores.lock().unwrap().insert(score.key.clone(), score.score);
        } else {
            * allscores.lock().unwrap().get_mut(&score.key).unwrap() = score.score; 
        }


        if self.lowest_hs > score.score || self.scorelist.len() < 10 {
            /* create or update entry */ 
            if !self.scorelist.contains_key(&score.key) {
                self.scorelist.insert(score.key, (score.score, String::from_utf8_lossy(score.name.as_slice()).to_string()));
            } else {
                * self.scorelist.get_mut(&score.key).unwrap() = (score.score, String::from_utf8_lossy(score.name.as_slice()).to_string());
            }

            let mut tmp = Vec::new();

            for (_,entry) in &self.scorelist {
                tmp.push(entry.clone());
            }
            
            /* order entries */
            tmp.sort_by(|a,b| a.0.partial_cmp(&b.0).unwrap());


            let mut lboard = board.lock().unwrap();
            lboard.clear();

            /* write */
            let mut count = 1;
            for entry in tmp {
                lboard.deref_mut().push_str(&count.to_string());
                lboard.deref_mut().push_str(". " );
                lboard.deref_mut().push_str(&entry.0.to_string());
                lboard.deref_mut().push_str( "  ");
                lboard.deref_mut().push_str(&entry.1);
                lboard.deref_mut().push('\n');
                count+=1;
            }
        }
    }
}