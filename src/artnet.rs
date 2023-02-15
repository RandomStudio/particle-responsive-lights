use artnet_protocol::*;
use nannou::prelude::{map_range, ToPrimitive};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::particles::Particle;

pub struct ArtNetInterface {
    socket: UdpSocket,
    destination: SocketAddr,
}

pub enum ArtNetMode {
    Broadcast,
    /// Specify destination address only
    UnicastAllInterfaces(SocketAddr),
    /// Specify from (interface) + to (destination) addresses
    UnicastSpecifyInterface(SocketAddr, SocketAddr),
}

impl ArtNetInterface {
    pub fn new(mode: ArtNetMode) -> Self {
        match mode {
            ArtNetMode::Broadcast => {
                let socket = UdpSocket::bind((String::from("0.0.0.0"), 6455)).unwrap();
                let broadcast_addr = ("255.255.255.255", 6454)
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap();
                socket.set_broadcast(true).unwrap();
                ArtNetInterface {
                    socket,
                    destination: broadcast_addr,
                }
            }
            ArtNetMode::UnicastAllInterfaces(destination) => {
                let socket = UdpSocket::bind((String::from("0.0.0.0"), 6455)).unwrap();
                socket.set_broadcast(false).unwrap();
                ArtNetInterface {
                    socket,
                    destination,
                }
            }
            ArtNetMode::UnicastSpecifyInterface(src, destination) => {
                let socket = UdpSocket::bind(src).unwrap();

                socket.set_broadcast(false).unwrap();
                ArtNetInterface {
                    socket,
                    destination,
                }
            }
        }
    }

    pub fn update(&self, particles: &[Particle]) {
        let mut channels: Vec<u8> = vec![];
        let channels_per_fixture = 3;

        for p in particles {
            for _i in 0..channels_per_fixture {
                channels.push(map_range(p.brightness, 0., 1., 0., 255.).to_u8().unwrap());
            }
        }

        let command = ArtCommand::Output(Output {
            data: channels.into(),
            ..Output::default()
        });
        let buff = command.write_to_buffer().unwrap();
        self.socket.send_to(&buff, self.destination).unwrap();
    }
}
