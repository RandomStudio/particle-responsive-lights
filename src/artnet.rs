use artnet_protocol::*;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub struct ArtNetInterface {
    socket: UdpSocket,
    broadcast_addr: SocketAddr,
}

impl ArtNetInterface {
    pub fn new() -> Self {
        let socket = UdpSocket::bind(("0.0.0.0", 6455)).unwrap();
        let broadcast_addr = ("255.255.255.255", 6454)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        socket.set_broadcast(true).unwrap();
        ArtNetInterface {
            socket,
            broadcast_addr,
        }
    }

    pub fn update(&self) {
        // let buff = ArtCommand::Poll(Poll::default()).write_to_buffer().unwrap();
        let command = ArtCommand::Output(Output {
            data: vec![128, 0, 255, 64, 5].into(),
            ..Output::default()
        });
        let buff = command.write_to_buffer().unwrap();
        self.socket.send_to(&buff, self.broadcast_addr).unwrap();
    }
}
