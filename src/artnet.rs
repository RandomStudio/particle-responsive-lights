use artnet_protocol::*;
use log::debug;
use nannou::prelude::{map_range, ToPrimitive};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use tween::Tweener;

use crate::particles::Particle;

type LUT = [u8; 256];

pub struct ArtNetInterface {
    socket: UdpSocket,
    destination: SocketAddr,
    brightness_mapping: Option<LUT>,
}

pub enum ArtNetMode {
    Broadcast,
    /// Specify from (interface) + to (destination) addresses
    Unicast(SocketAddr, SocketAddr),
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
                    brightness_mapping: None,
                }
            }
            ArtNetMode::Unicast(src, destination) => {
                let socket = UdpSocket::bind(src).unwrap();

                socket.set_broadcast(false).unwrap();
                ArtNetInterface {
                    socket,
                    destination,
                    brightness_mapping: None,
                }
            }
        }
    }

    pub fn create_brightness_mapping(&mut self) {
        let mut lookup: LUT = [0; 256];
        let mut tweener = Tweener::cubic_in(0., 1.0, 256);
        for i in 0..=255 {
            let output = tweener.move_by(1);
            // let output = i.try_into().unwrap();
            let output_rounded = (output * 255.).to_u8().unwrap_or(0);
            debug!("input level {i} -> {output_rounded} (from {output})");
            lookup[i] = output_rounded;
        }
        self.brightness_mapping = Some(lookup);
    }

    pub fn update(&self, particles: &[Particle]) {
        let mut channels: Vec<u8> = vec![];
        let channels_per_fixture = 3;

        for p in particles {
            for _i in 0..channels_per_fixture {
                if let Some(brightness) = map_range(p.brightness(), 0., 1., 0., 255.).to_u8() {
                    match self.brightness_mapping {
                        Some(lookup) => {
                            channels.push(lookup[brightness.to_usize().unwrap()]);
                        }
                        None => {
                            channels.push(brightness);
                        }
                    }
                }
            }
        }

        let command = ArtCommand::Output(Output {
            port_address: 0.into(),
            data: channels.into(),
            ..Output::default()
        });
        let buff = command.write_to_buffer().unwrap();
        self.socket.send_to(&buff, self.destination).unwrap();
    }
}
