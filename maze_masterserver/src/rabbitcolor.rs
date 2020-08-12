use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize,Clone,Copy)]
pub struct RabbitColor {
    raw : u8
}