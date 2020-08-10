use super::Logic;

#[derive(PartialEq)]
pub enum DispatchType {
    PosUpdate,
    Emoji,
}

pub enum PacketParseResult <T> {
    OkDispatch(T),
    Ignore,
    HardError,
}

pub trait Dispatch {
    fn dispatch(&self, logic : &Logic) -> Option<(Vec<u8>,DispatchType)>;
}