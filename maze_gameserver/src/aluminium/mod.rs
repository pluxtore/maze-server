pub mod clientman;
pub mod antiddos;
mod listener;
mod globaleventqueue;

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use clientman::ClientManager;

pub struct Server {
    listeners: Vec<Arc<Mutex<listener::Listener>>>,
    thread_handles: Vec<JoinHandle<()>>,
    ports: (u16, u16), // start,end
    feedback_rx: Arc<Mutex<Receiver<String>>>,
}

impl Server {
    pub fn start(ports: (u16, u16)) -> Self {
        let (feedback_tx, feedback_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (sender_inbound, receiver_inbound) = mpsc::channel();

        let mut thread_handles = Vec::new();
        let mut listeners = Vec::new();

        for port in ports.0..ports.1 {
            listeners.push(Arc::new(Mutex::new(listener::Listener::new(
                port,
                sender_inbound.clone(),
            ))))
        }

        for list in &listeners {
            let tmp = list.clone();
            thread_handles.push(thread::spawn(move || {
                tmp.clone().lock().unwrap().run();
            }))
        }
        thread_handles.push(std::thread::spawn(move || {
            ClientManager::new(receiver_inbound, feedback_tx).run();
        }));

        Self {
            listeners: listeners,
            thread_handles: thread_handles,
            ports: ports,
            feedback_rx: Arc::new(Mutex::new(feedback_rx)),
        }
    }
}
