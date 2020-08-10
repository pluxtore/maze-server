use crate::gamelogic::{self,Logic,clientkey::ClientKey};
use std::collections::{HashMap};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use super::globaleventqueue::GlobalEventQueue;

pub struct Message {
    pub socket: UdpSocket,
    pub peer_addr: SocketAddr,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(socket: UdpSocket, peer_addr: SocketAddr, payload: Vec<u8>) -> Self {
        Self {
            socket: socket,
            peer_addr: peer_addr,
            payload: payload,
        }
    }
}


impl Clone for Message {
    fn clone(&self) -> Self {
        let newsocket = match self.socket.try_clone() {
            Ok(e) => {
                e
            }
            _ => {
                log::error!("fatal error : could not clone socket");
                panic!("fatal error : could not clone socket");
            }
        };


        Self {
            socket : newsocket,
            peer_addr : self.peer_addr.clone(),
            payload : self.payload.clone(),
        }
    }
}

pub struct ClientManager {
    receiver: Receiver<Message>,
    pool: ClientPool,
    feedback: Sender<String>,
}
impl ClientManager {
    /*constructor for client manager*/
    pub fn new(receiver: Receiver<Message>, feedback: Sender<String>) -> Self {
        log::info!("new ClientManager instance created");
        Self {
            receiver: receiver,
            pool: ClientPool::new(),
            feedback: feedback,
        }
    }
    /* runmethod that is executed as worker thread, does not terminate ! */
    pub fn run(&mut self) -> ! {
        log::info!("ClientManager entering runmethod");
        loop {
            for msg in self.receiver.recv() {
                loop {
                    match self.pool.shutdown_endpoint.try_recv() {
                        Ok(e) => {
                            self.pool.client_endpoints.remove(&e);
                            log::info!("removed client {:x?} out of in-memory dict", e)
                        }
                        _ => break,
                    };
                }
                self.pool.route(msg);
            }
        }
    }
}
pub struct ClientPool {
    /* route packets to clients */
    client_endpoints: HashMap<ClientKey, Sender<Message>>,
    global_event_queue: Arc<Mutex<GlobalEventQueue>>,
    shutdown_endpoint: Receiver<ClientKey>,
    shutdown_sender: Sender<ClientKey>,
}

impl ClientPool {
    pub fn new() -> Self {
        log::info!("new ClientPool instance created");
        let (tx, rx) = mpsc::channel();
        Self {
            client_endpoints: HashMap::new(),
            global_event_queue: Arc::new(Mutex::new(GlobalEventQueue::new())),
            shutdown_endpoint: rx,
            shutdown_sender: tx,
        }
    }
    pub fn route(&mut self, message: Message) {
        let payload_tmp = message.payload.clone();
        match self.client_endpoints.get(&Logic::pkt_get_client_key(&payload_tmp)) {
            Some(e) => {
                match e.send(message) {
                    Ok(_) => (),
                    Err(_) => {
                        log::warn!("could not route packet to client receiver, shutting down endpoint");
                        self.client_endpoints.remove(&Logic::pkt_get_client_key(&payload_tmp));
                        return;
                    }
                };
            }
            None => {
                self.add_client(message);
            }
        }
    }
    fn add_client(&mut self, message: Message) {
        let (tx, rx) = mpsc::channel();
        let peer_addr = message.peer_addr;

        let logic = match Logic::new(message) {
            Some(e) => e,
            None => {
                log::warn!("{} sent invalid handshake", peer_addr);
                return
            },
        };

        let key = logic.get_identitiy().get_key().clone();
        let _key = key.clone();

        let geq = self.global_event_queue.clone();
        self.client_endpoints.insert( _key, tx);
        geq.lock().unwrap().add_client(logic.get_identitiy().get_key().clone());

        client_init(
            logic,
            rx,
            geq,
            self.shutdown_sender.clone(),
        );
        
    }
}

pub fn client_init(
    mut logic: Logic,
    receiver: Receiver<Message>,
    geq: Arc<Mutex<GlobalEventQueue>>,
    shutdown_tx: Sender<ClientKey>,
) {
    thread::spawn(move || {
        /* event loop*/
        let mut timeout = 0;

        loop {
            let update = receiver.recv_timeout(Duration::from_millis(timeout));

            if update.is_ok() {
                let update = update.unwrap();

                if update.peer_addr.ip()!=logic.get_identitiy().peer_addr.ip() {
                    break;
                }

                match logic.inbound_parse(update) {
                    ParseFeedback::NewEvent(inbound_event) => {
                        geq.lock().unwrap().push_update(inbound_event, logic.get_identitiy().get_key());
                    },
                    ParseFeedback::Ignore => continue,
                    ParseFeedback::PullNewEvents => (),
                    ParseFeedback::ForceTermination => break
                };

            } else {
                if timeout != 0 {
                    break;
                }
            }
            
            logic.outbound_create(geq.lock().unwrap().pull_updates(&logic.get_identitiy().get_key()));

            timeout = *gamelogic::client_thread_timeout;
        }

        log::info!(
            "client {:x?} suspending activity, thread terminating",
            logic.get_identitiy().get_key()
        );

        logic.shutdown();
        geq.lock().unwrap().delete_client(logic.get_identitiy().get_key());

        match shutdown_tx.send(logic.get_identitiy().get_key().clone()) {
            Ok(_e) => log::info!(
                "client {:x?} sending shutdown command to router",
                logic.get_identitiy().get_key()
            ),
            Err(_e) => {
                log::warn!("client {:x?} could not send shutdown command, this will very likely cause errors when reconnecting", logic.get_identitiy().get_key());
            }
        }
    });
}

pub enum ParseFeedback <T> {
    NewEvent(T),
    PullNewEvents,
    Ignore,
    ForceTermination,
}