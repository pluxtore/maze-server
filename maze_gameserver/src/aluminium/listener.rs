/* listens on one udp port to put packets into FIFO queue for arriving packets */
use crate::aluminium::clientman::Message;
use crate::gamelogic::Logic;
use std::ops::Add;
use std::sync::mpsc::Sender;

pub struct Listener {
    socket: std::net::UdpSocket,
    sender: Sender<Message>,
    port: u16,
}
impl Listener {
    /* constructor for the listener */
    pub fn new(port: u16, sender: Sender<Message>) -> Self {
        let bindaddr = String::from("0.0.0.0:").add(&port.to_string());
        let tmp = match std::net::UdpSocket::bind(bindaddr) {
            Err(_e) => {
                log::error!(
                    "{}",
                    String::from("could not bind to port ").add(&port.to_string())
                );
                panic!("could not bind to port")
            }
            Ok(e) => e,
        };
        log::info!("new instance on port {} created ", port);
        Self {
            socket: tmp,
            sender: sender,
            port: port,
        }
    }
    /* runmethod that is executed as worker thread, does not terminate ! */
    pub fn run(&self) -> ! {
        let mut recvbuffer: [u8; 256] = [0; 256];
        log::info!("instance for port {} entering runmethod", self.port);
        loop {
            for (nbytes, src) in self.socket.recv_from(&mut recvbuffer) {
                if nbytes < 10  || nbytes > 48 {
                    continue;
                }
                let cloned_socket = match self.socket.try_clone() {
                    Ok(e) => e,
                    Err(_e) => {
                        log::error!(
                            "could not clone socket on local port {}, ignoring packet",
                            self.port
                        );
                        continue;
                    }
                };

                match self.sender.send(Message::new(
                    cloned_socket,
                    src,
                    Logic::decode_pkt(&recvbuffer[0..nbytes]),
                )) {
                    Ok(_e) => (),
                    Err(_e) => {
                        log::warn!("failed to send payload into FIFO mpsc, out of memory?");
                        continue;
                    }
                };
            }
        }
    }
}
