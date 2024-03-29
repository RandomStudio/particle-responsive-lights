use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use clap::Parser;
use log::{debug, error, info, warn};
use nannou::prelude::*;
use nannou_egui::Egui;
use serde::{Deserialize, Serialize};
use tween::*;

use crate::artnet::{ArtNetInterface, ArtNetMode};
use crate::particles::build_layout;
use crate::particles::Particle;
use crate::tether::TetherAgent;

use strum_macros::Display;
use strum_macros::EnumIter;

const DEFAULT_SETTINGS_FILE_PATH: &str = "./settings.json";

pub const DEFAULT_WINDOW_W: u32 = 1280;
pub const DEFAULT_WINDOW_H: u32 = 600;

pub const DEFAULT_COUNT: usize = 14;
pub const DEFAULT_ORDER: [usize; DEFAULT_COUNT] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

const DEFAULT_THICKNESS: f32 = 15.;
const DEFAULT_LENGTH: f32 = 250.;
const DEFAULT_ATTACK_DURATION: usize = 2000;
const DEFAULT_RELEASE_DURATION: usize = 2000;

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

const DEFAULT_BRIGHTNESS_MAPPING: EaseStyle = EaseStyle::Linear;

const DEFAULT_ARTNET_HERTZ: usize = 44;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    /// Flag to disable Tether connection
    #[arg(long = "tether.disable")]
    tether_disable: bool,

    /// The IP address of the Tether MQTT broker (server)
    #[arg(long = "tether.host", default_value_t=TETHER_HOST)]
    tether_host: std::net::IpAddr,

    /// Flag to enable ArtNet broadcast mode (good for development)
    #[arg(long = "artnet.broadcast")]
    artnet_broadcast: bool,

    /// IP address for ArtNet source interface (ignored if broadcast enabled)
    #[arg(long = "artnet.interface", default_value_t=UNICAST_SRC)]
    pub unicast_src: std::net::IpAddr,

    /// IP address for ArtNet destination node (ignored if broadcast enabled)
    #[arg(long = "artnet.destination", default_value_t=UNICAST_DST)]
    pub unicast_dst: std::net::IpAddr,

    /// Update frequency, in Hertz, for sending ArtNet data (gets converted to ms)
    #[arg(long = "artnet.freq", default_value_t=DEFAULT_ARTNET_HERTZ)]
    pub artnet_update_frequency: usize,

    /// Flag to enable ArtNet 16-bit resolution
    #[arg(long = "artnet.high")]
    artnet_high_resolution: bool,

    /// How many channels per pixel, e.g. RGBW=4, L(16)=1
    #[arg(long = "artnet.pixelChannels", default_value_t = 1)]
    artnet_channels_per_fixture: usize,

    /// Start with minimum graphics (no chime graphics, no GUI)
    #[arg(long = "minGraphics")]
    pub use_min_graphics: bool,

    /// Ignore the settings.json file, even if it exists; apply hard-coded defaults instead
    #[arg(long = "ignoreFile")]
    ignore_settings_file: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhaseSettings {
    pub duration: usize,
    pub style: EaseStyle,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransmissionSettings {
    pub max_range: f32,
    pub max_delay: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub chimes_count: usize,
    pub channels_per_pixel: usize,
    pub artnet_high_res: bool,
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
    pub use_min_graphics: bool,
    #[serde(skip)]
    pub fixture_order_string: String,
    #[serde(skip)]
    pub fixture_order_editing: bool,
    pub fixture_order: [usize; DEFAULT_COUNT],
    pub artnet_update_interval: u64,
}

impl Settings {
    pub fn save(&self) -> Result<(), ()> {
        let file_path = DEFAULT_SETTINGS_FILE_PATH;
        let text = serde_json::to_string_pretty(self).expect("Failed to serialise Settings");
        match std::fs::write(file_path, text) {
            Ok(()) => {
                info!("Wrote settings to file {file_path}");
                Ok(())
            }
            Err(e) => {
                error!("Error writing settings to file: {:?}", e);
                Err(())
            }
        }
    }

    pub fn load(&mut self) -> Result<(), ()> {
        let file_path = DEFAULT_SETTINGS_FILE_PATH;
        match std::fs::read_to_string(file_path) {
            Ok(text) => match serde_json::from_str::<Settings>(&text) {
                Ok(data) => {
                    *self = Settings { ..data };
                    info!("Loaded settings from file {file_path} ok");
                    debug!("Loaded: {:?}", self);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to parse settings data: {e}");
                    Err(())
                }
            },
            Err(e) => {
                error!("Error when loading settings file: {e}");
                Err(())
            }
        }
    }
}

pub struct Model {
    pub window_id: WindowId,
    pub particles: Vec<Particle>,
    pub mouse_position: Point2,
    pub egui: Egui,
    pub settings: Settings,
    pub artnet: ArtNetInterface,
    pub tether: TetherAgent,
    pub last_artnet_sent: std::time::SystemTime,
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

        let mut settings = Settings {
            chimes_count: DEFAULT_COUNT,
            chime_thickness: DEFAULT_THICKNESS,
            chime_length: DEFAULT_LENGTH,
            channels_per_pixel: cli.artnet_channels_per_fixture,
            artnet_high_res: cli.artnet_high_resolution,
            attack_settings: PhaseSettings {
                duration: DEFAULT_ATTACK_DURATION,
                style: EaseStyle::SineBoth,
            },
            release_settings: PhaseSettings {
                duration: DEFAULT_RELEASE_DURATION,
                style: EaseStyle::SineBoth,
            },
            transmission_settings: TransmissionSettings {
                max_range: DEFAULT_TRANSMISSION_RANGE,
                max_delay: DEFAULT_TRANSMISSION_DELAY,
            },
            show_brightness_indicator: DEFAULT_SHOW_B_INDICATOR,
            show_chime_index: DEFAULT_SHOW_INDEX,
            trigger_full_brightness: DEFAULT_TRIGGER_FULL,
            trigger_by_order: DEFAULT_TRIGGER_BY_ORDER,
            mouse_enable: !cli.use_min_graphics,
            mouse_brightness_value: 1.0,
            resting_brightness: 0.,
            lights_lookup_mapping: DEFAULT_BRIGHTNESS_MAPPING,
            fixture_order: DEFAULT_ORDER,
            fixture_order_string: fixture_array_to_string(&DEFAULT_ORDER),
            fixture_order_editing: false,
            use_min_graphics: cli.use_min_graphics,
            artnet_update_interval: (1000. / cli.artnet_update_frequency.to_f32())
                .to_u64()
                .unwrap(),
        };

        if cli.ignore_settings_file {
            warn!("Asked to ignore settings file from CLI; hard-coded defaults will apply");
        } else {
            match settings.load() {
                Ok(()) => {
                    info!("Settings loaded OK from file");
                }
                Err(()) => {
                    warn!("Settings could not be loaded from file; maybe create one?");
                }
            }
        }

        Model {
            window_id,
            particles: build_layout(
                DEFAULT_COUNT,
                DEFAULT_WINDOW_W.to_f32() * DEFAULT_WIDTH_RATIO,
                DEFAULT_WINDOW_H.to_f32() * DEFAULT_HEIGHT_RATIO,
                &settings.fixture_order,
            ),
            mouse_position: Point2::new(0., 0.),
            settings,
            egui,
            artnet,
            tether,
            last_artnet_sent: std::time::SystemTime::now(),
        }
    }
}

pub fn fixture_array_to_string(arr: &[usize; DEFAULT_COUNT]) -> String {
    let mut s = String::from("");
    arr.iter().enumerate().for_each(|(i, x)| {
        if i > 0 {
            s += ",";
        }
        s += &format!("{}", x);
    });
    s
}

pub fn fixture_string_to_array(s: &str) -> Result<[usize; DEFAULT_COUNT], ()> {
    let parts = s.split(",");
    let mut arr = Vec::new();
    parts.into_iter().for_each(|x| {
        if let Ok(num) = x.parse::<usize>() {
            arr.push(num);
        } else {
            error!("Failed to parse '{}' as number", x);
        }
    });
    if arr.len() == DEFAULT_COUNT {
        let as_array = arr
            .try_into()
            .unwrap_or_else(|_v| panic!("String to array conversion failed"));
        Ok(as_array)
    } else {
        Err(())
    }
}

// TODO: seems tedious to have to re-write all these enums
// but Box<dyn Tween<f32>> is difficult to impl PartialEQ for
// so UI / ComboBox is difficult
#[derive(PartialEq, Debug, EnumIter, Display, Serialize, Deserialize)]
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
