use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use clap::Parser;
use log::warn;
use nannou::prelude::*;
use nannou_egui::Egui;
use tween::*;

use crate::artnet::{ArtNetInterface, ArtNetMode};
use crate::particles::build_layout;
use crate::particles::Particle;
use crate::tether::TetherAgent;

use strum_macros::Display;
use strum_macros::EnumIter;

pub const DEFAULT_WINDOW_W: u32 = 1280;
pub const DEFAULT_WINDOW_H: u32 = 600;

const DEFAULT_COUNT: usize = 14;
// TODO: this should be not be hard-coded; maybe a string => hashmap, length checked at runtime?
// pub const DEFAULT_ORDER: [usize; DEFAULT_COUNT] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
// pub const DEFAULT_ORDER: [usize; DEFAULT_COUNT] = [8, 9, 7, 6, 10, 11, 13, 1, 5, 12, 0, 4, 2, 3];
pub const DEFAULT_ORDER: [usize; DEFAULT_COUNT] = [9, 8, 7, 10, 6, 11, 13, 1, 5, 12, 0, 2, 4, 3];

const DEFAULT_THICKNESS: f32 = 15.;
const DEFAULT_LENGTH: f32 = 250.;
const DEFAULT_ATTACK_DURATION: usize = 200;
const DEFAULT_RELEASE_DURATION: usize = 1000;

const DEFAULT_SHOW_B_INDICATOR: bool = true;
const DEFAULT_SHOW_INDEX: bool = true;

const DEFAULT_TRIGGER_FULL: bool = false;
const DEFAULT_TRIGGER_BY_ORDER: bool = true;

pub const DEFAULT_WIDTH_RATIO: f32 = 0.6;
pub const DEFAULT_HEIGHT_RATIO: f32 = 0.2;

const DEFAULT_TRANSMISSION_RANGE: f32 = 0.15;
const DEFAULT_TRANSMISSION_DELAY: i64 = 500;

const TETHER_HOST: std::net::IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const UNICAST_SRC: std::net::IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 102));
const UNICAST_DST: std::net::IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

const DEFAULT_BRIGHTNESS_MAPPING: EaseStyle = EaseStyle::QuadIn;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    /// Whether to disable Tether connection
    #[arg(long = "tether.disable")]
    tether_disable: bool,

    /// The IP address of the Tether MQTT broker (server)
    #[arg(long = "tether.host", default_value_t=TETHER_HOST)]
    tether_host: std::net::IpAddr,

    /// Whether to enable ArtNet broadcast mode (good for development)
    #[arg(long = "artnet.broadcast")]
    artnet_broadcast: bool,

    /// IP address for ArtNet source interface (ignored if broadcast enabled)
    #[arg(long = "artnet.interface", default_value_t=UNICAST_SRC)]
    pub unicast_src: std::net::IpAddr,

    /// IP address for ArtNet destination node (ignored if broadcast enabled)
    #[arg(long = "artnet.destination", default_value_t=UNICAST_DST)]
    pub unicast_dst: std::net::IpAddr,
}

pub struct PhaseSettings {
    pub duration: usize,
    pub style: EaseStyle,
}

pub struct TransmissionSettings {
    pub max_range: f32,
    pub max_delay: i64,
}

pub struct Settings {
    pub chimes_count: usize,
    pub show_brightness_indicator: bool,
    pub show_chime_index: bool,
    pub chime_thickness: f32,
    pub chime_length: f32,
    pub attack_settings: PhaseSettings,
    pub release_settings: PhaseSettings,
    pub transmission_settings: TransmissionSettings,
    pub trigger_full_brightness: bool,
    pub trigger_by_order: bool,
    pub mouse_enable: bool,
    pub mouse_brightness_value: f32,
    pub resting_brightness: f32,
    pub lights_lookup_mapping: EaseStyle,
}

pub struct Model {
    pub window_id: WindowId,
    pub particles: Vec<Particle>,
    pub mouse_position: Point2,
    pub egui: Egui,
    pub settings: Settings,
    pub artnet: ArtNetInterface,
    pub tether: TetherAgent,
}

impl Model {
    pub fn defaults(window_id: WindowId, egui: Egui, cli: &Cli) -> Self {
        let mut tether = TetherAgent::new(cli.tether_host);
        if !cli.tether_disable {
            tether.connect();
        } else {
            warn!("Tether connection disabled")
        }

        let mut artnet = {
            if cli.artnet_broadcast {
                ArtNetInterface::new(ArtNetMode::Broadcast)
            } else {
                ArtNetInterface::new(ArtNetMode::Unicast(
                    SocketAddr::from((cli.unicast_src, 6454)),
                    SocketAddr::from((cli.unicast_dst, 6454)),
                ))
            }
        };
        artnet.create_brightness_mapping(&DEFAULT_BRIGHTNESS_MAPPING);

        Model {
            window_id,
            particles: build_layout(
                DEFAULT_COUNT,
                DEFAULT_WINDOW_W.to_f32() * DEFAULT_WIDTH_RATIO,
                DEFAULT_WINDOW_H.to_f32() * DEFAULT_HEIGHT_RATIO,
            ),
            mouse_position: Point2::new(0., 0.),
            settings: Settings {
                chimes_count: DEFAULT_COUNT,
                chime_thickness: DEFAULT_THICKNESS,
                chime_length: DEFAULT_LENGTH,
                attack_settings: PhaseSettings {
                    duration: DEFAULT_ATTACK_DURATION,
                    style: EaseStyle::SineBoth,
                },
                release_settings: PhaseSettings {
                    duration: DEFAULT_RELEASE_DURATION,
                    style: EaseStyle::BounceIn,
                },
                transmission_settings: TransmissionSettings {
                    max_range: DEFAULT_TRANSMISSION_RANGE,
                    max_delay: DEFAULT_TRANSMISSION_DELAY,
                },
                show_brightness_indicator: DEFAULT_SHOW_B_INDICATOR,
                show_chime_index: DEFAULT_SHOW_INDEX,
                trigger_full_brightness: DEFAULT_TRIGGER_FULL,
                trigger_by_order: DEFAULT_TRIGGER_BY_ORDER,
                mouse_enable: true,
                mouse_brightness_value: 1.0,
                resting_brightness: 0.,
                lights_lookup_mapping: DEFAULT_BRIGHTNESS_MAPPING,
            },
            egui,
            artnet,
            tether,
        }
    }
}

// TODO: seems tedious to have to re-write all these enums
// but Box<dyn Tween<f32>> is difficult to impl PartialEQ for
// so UI / ComboBox is difficult
#[derive(PartialEq, Debug, EnumIter, Display)]
pub enum EaseStyle {
    Linear,
    BounceIn,
    BounceOut,
    BounceBoth,
    SineIn,
    SineOut,
    SineBoth,
    ElasticIn,
    ElasticOut,
    ElasticBoth,
    QuadIn,
    QuadOut,
    QuadBoth,
    ExpoIn,
    ExpoOut,
    ExpoBoth,
    CubicIn,
    CubicOut,
    CubicBoth,
    CircIn,
    CircOut,
    CircBoth,
}

// TODO: new allocation for every new animation? A little inefficient
// Could be cached somehow, e.g. if the tween already exists.
pub fn get_new_tween(style: &EaseStyle) -> Box<dyn Tween<f32>> {
    match style {
        EaseStyle::Linear => Box::new(Linear),
        EaseStyle::BounceIn => Box::new(BounceIn),
        EaseStyle::BounceOut => Box::new(BounceOut),
        EaseStyle::BounceBoth => Box::new(BounceInOut),
        EaseStyle::SineIn => Box::new(SineIn),
        EaseStyle::SineOut => Box::new(SineOut),
        EaseStyle::SineBoth => Box::new(SineInOut),
        EaseStyle::QuadIn => Box::new(QuadIn),
        EaseStyle::QuadOut => Box::new(QuadOut),
        EaseStyle::QuadBoth => Box::new(QuadInOut),
        EaseStyle::ExpoIn => Box::new(ExpoIn),
        EaseStyle::ExpoOut => Box::new(ExpoOut),
        EaseStyle::ExpoBoth => Box::new(ExpoInOut),
        EaseStyle::CubicIn => Box::new(CubicIn),
        EaseStyle::CubicOut => Box::new(CubicOut),
        EaseStyle::CubicBoth => Box::new(CubicInOut),
        EaseStyle::CircIn => Box::new(CircIn),
        EaseStyle::CircOut => Box::new(CircInOut),
        EaseStyle::CircBoth => Box::new(CircInOut),
        EaseStyle::ElasticIn => Box::new(ElasticIn),
        EaseStyle::ElasticOut => Box::new(ElasticInOut),
        EaseStyle::ElasticBoth => Box::new(ElasticInOut),
    }
}
