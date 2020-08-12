use serde::Deserialize;
use super::ClientKey;

#[derive(Deserialize)]
pub struct Score {
    pub key : ClientKey,
    pub name : Vec<u8>,
    pub score : f32,
}