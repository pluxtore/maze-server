use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize,Clone,Copy)]
pub struct RabbitColor {
    raw : u8
}


impl RabbitColor {
    pub fn new() -> Self {
        Self { raw : rand::random::<u8>() % 5}
    }

    pub fn make_white(&mut self) {
        self.raw=5;
    }

    pub fn get_raw(&self) -> u8 {
        self.raw
    }
}