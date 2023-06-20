use artnet_protocol::*;
use log::debug;
use nannou::prelude::{map_range, ToPrimitive};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use tween::Tweener;

use crate::{
    particles::Particle,
    settings::{get_new_tween, EaseStyle},
};

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

const SIXTEEN_MAX: f32 = u16::MAX as f32;

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

    pub fn create_brightness_mapping(&mut self, ease_style: &EaseStyle) {
        let mut lookup: LUT = [0; 256];

        // let mut tweener = Tweener::quad_in(0., 1.0, 255);
        let tween = get_new_tween(&ease_style);
        let mut tweener = Tweener::new(0., 1.0, 255, tween);

        for i in 0..=255 {
            let output = tweener.move_to(i);
            // let output = i.try_into().unwrap();
            let output_rounded = (output * 255.).to_u8().unwrap_or(0);
            debug!("input level {i} -> {output_rounded} (from {output})");
            lookup[i] = output_rounded;
        }
        self.brightness_mapping = Some(lookup);
    }

    pub fn update(&self, particles: &[Particle], channels_per_fixture: usize, use_high_res: bool) {
        let mut channels: Vec<u8> = vec![];
        if use_high_res {
            for p in particles {
                for _i in 0..channels_per_fixture {
                    let l_sixteen: u16 = (p.brightness() * SIXTEEN_MAX) as u16;
                    let [c1, c2] = convert_u16_to_two_u8s_be(l_sixteen);
                    channels.push(c1);
                    channels.push(c2);
                }
            }
        } else {
            for p in particles {
                for _i in 0..channels_per_fixture {}
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

fn convert_u16_to_two_u8s_be(integer: u16) -> [u8; 2] {
    [(integer >> 8) as u8, integer as u8]
}
