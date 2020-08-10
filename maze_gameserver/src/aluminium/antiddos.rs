use std::net::IpAddr;
use std::collections::HashMap;
use std::process::Command;


struct AntiDdos {
    suspicious : HashMap<IpAddr,u8>,
}

impl AntiDdos {
    pub fn evaluate(&mut self,ip : IpAddr) {
        if self.suspicious.contains_key(&ip) {
                *self.suspicious.get_mut(&ip).unwrap() += 1;
                if self.suspicious.get_mut(&ip).unwrap() > &mut 100 {
                    let mut cmd = "block ".to_string();
                    cmd.push_str(ip.to_string().as_str());
                    Command::new(cmd);
                }
        }
    }
}