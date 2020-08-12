#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

mod clientkey;
mod characterstate;
mod score;
mod unlocks;
mod rabbitcolor;

use std::sync::Mutex;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use clientkey::ClientKey;
use score::Score;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

lazy_static! {
    pub static ref board : Mutex<String> = Mutex::new(String::new());
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

fn main() {
    std::thread::spawn(move || {
        Handler::new().handler();        
    });
    rocket::ignite().mount("/", routes![api]).launch();
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
            let (socket,_) = match listener.accept() {
                Ok(e) => e,
                _ => continue,
            };
            self.process(socket);
        }
    }

    fn process(&mut self,mut stream : TcpStream) {
        let mut buf : [u8;500] = [0;500];
        let len  = match stream.read(&mut buf) {
            Ok(e) => e,
            _ => return,
        };
        let score : Score = bincode::deserialize(&buf[..len]).unwrap();

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
            let count = 1;
            for entry in tmp {
                lboard.deref_mut().push_str(&count.to_string());
                lboard.deref_mut().push_str(". " );
                lboard.deref_mut().push_str(&entry.0.to_string());
                lboard.deref_mut().push_str( "  ");
                lboard.deref_mut().push_str(&entry.1);
                lboard.deref_mut().push('\n');
            }
        }
    }
}