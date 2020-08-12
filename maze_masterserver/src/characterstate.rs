use super::unlocks::Unlocks;
use super::rabbitcolor::RabbitColor;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CharacterState {
    name: Vec<u8>,
    position: (i32, i32, i32),
    is_alive: bool,
    is_new: bool,
    pub unlocks: Unlocks,
    pub color : RabbitColor,
}

