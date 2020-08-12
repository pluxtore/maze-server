use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize,Copy)]
pub struct Unlocks {
    raw: u8,
}

impl Unlocks {
    pub fn new() -> Self {
        Self { raw: 0 }
    }

    pub fn get(&self,index : u8) -> bool {
        ( self.raw>>index ) & 1 == 1 
    }

    pub fn get_raw(&self) -> u8 {
        self.raw
    }

    pub fn get_total(&self) -> i32 {
        let mut ret = 0;
        for i in 0..7 {
            if self.get(i) {
                ret+=1;
            }
        }
        ret
    }

    pub fn set(&mut self,index : u8) {
        self.raw = self.raw | (1<<index)
    }
}