use serde::Serialize;
use super::ClientKey;

#[derive(Serialize)]
pub struct Score {
    key : ClientKey,
    name : Vec<u8>,
    score : f32,
}

impl Score {
    pub fn new(key : ClientKey, name : Vec<u8>, score_f32 : f32) -> Self {
        Self {
            key : key,
            name : name,
            score : score_f32,
        }
    }
}
