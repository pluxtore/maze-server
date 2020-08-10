use std::collections::{VecDeque,HashMap};
use crate::gamelogic::clientkey::ClientKey;
use crate::gamelogic::types::*;
use std::sync::{Mutex,Arc};

pub struct GlobalEventQueue {
    client_queues: HashMap<ClientKey, VecDeque<Arc<Mutex<dyn Dispatch + Send>>>>,
}
impl GlobalEventQueue {
    pub fn new() -> Self {
        Self {
            client_queues: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, key : ClientKey) {
        self.client_queues.insert(key, VecDeque::new());
    }

    pub fn delete_client(&mut self, key : &ClientKey) {
        self.client_queues.remove(key);
    }

    pub fn push_update(&mut self, ev: Arc<Mutex<dyn Dispatch + Send>>, key : &ClientKey) {
        for (current_key, queue) in &mut self.client_queues {
            if current_key!=key {
                if queue.len()>25 {
                    queue.pop_front(); // ddos miltigation
                }
                queue.push_back(ev.clone());
            }
        }
    }
    pub fn pull_updates(&mut self, key: &ClientKey) -> Option<Vec<Arc<Mutex<dyn Dispatch + Send >>>> {
        let mut new_events : Vec<_> = Vec::new();
        
        match self.client_queues.get_mut(key) {
            Some(queue) => {
                match queue.is_empty() {
                    false => {
                        loop {
                            match queue.pop_front() {
                                Some(e) => {
                                    new_events.push(e);
                                }
                                None => break,
                            }
                        }
                    }
                    true => {
                        return None;
                    }
                }
            },
            None => {log::warn!("bad key");return None},
        };
        if !new_events.is_empty() {
        }
        Some(new_events)
    }
}
