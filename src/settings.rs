use std::time::Duration;

use nannou::prelude::*;
use nannou_egui::egui::{self, ComboBox};
use nannou_egui::Egui;
use tween::*;

use crate::particles::build_layout;
use crate::particles::Particle;

use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;

pub const DEFAULT_WINDOW_W: u32 = 1920;
pub const DEFAULT_WINDOW_H: u32 = 720;

const DEFAULT_COUNT: usize = 14;
const DEFAULT_THICKNESS: f32 = 15.;
const DEFAULT_LENGTH: f32 = 250.;
const DEFAULT_ATTACK_DURATION: usize = 300;
const DEFAULT_RELEASE_DURATION: usize = 2500;
const DEFAULT_SHOW_B_INDICATOR: bool = true;

pub const DEFAULT_WIDTH_RATIO: f32 = 0.6;
pub const DEFAULT_HEIGHT_RATIO: f32 = 0.2;

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
    pub chime_thickness: f32,
    pub chime_length: f32,
    pub attack_settings: PhaseSettings,
    pub release_settings: PhaseSettings,
    pub transmission_settings: TransmissionSettings,
}

pub struct Model {
    pub window_id: WindowId,
    pub particles: Vec<Particle>,
    pub mouse_position: Point2,
    pub egui: Egui,
    pub settings: Settings,
}

impl Model {
    pub fn defaults(window_id: WindowId, egui: Egui) -> Self {
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
                    max_range: DEFAULT_WINDOW_W.to_f32() * 0.2,
                    max_delay: 1000,
                },
                show_brightness_indicator: DEFAULT_SHOW_B_INDICATOR,
            },
            egui,
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

pub fn get_tween(style: &EaseStyle) -> Box<dyn Tween<f32>> {
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

pub fn build_ui(model: &mut Model, since_start: Duration, window_rect: Rect) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        // let settings = &mut model.settings;
        let Settings {
            chimes_count,
            show_brightness_indicator,
            chime_thickness,
            chime_length,
            attack_settings,
            release_settings,
            transmission_settings,
            ..
        } = &mut model.settings;

        ui.set_min_height(600.);

        ui.horizontal(|ui| {
            ui.label("Chimes count:");
            ui.add(egui::Slider::new(chimes_count, 1..=30));
            let current_count = chimes_count.to_owned();
            if ui.button("update").clicked() {
                model.particles = build_layout(
                    current_count,
                    window_rect.w() * DEFAULT_WIDTH_RATIO,
                    window_rect.h() * 0.2,
                )
            }
        });

        ui.separator();

        ui.checkbox(show_brightness_indicator, "Brightness indicator");

        ui.horizontal(|ui| {
            ui.label("Chimes thickness:");
            ui.add(egui::Slider::new(chime_thickness, 1. ..=200.).suffix("px"));
        });

        ui.horizontal(|ui| {
            ui.label("Chimes length:");
            ui.add(egui::Slider::new(chime_length, 10. ..=2000.).suffix("px"));
        });

        ui.separator();

        let PhaseSettings { duration, style } = attack_settings;

        ui.horizontal(|ui| {
            ui.label("Attack duration:");
            ui.add(egui::Slider::new(duration, 1..=10000).suffix("ms"));
        });

        ComboBox::from_label("Attack-phase Tween")
            .selected_text(style.to_string())
            .show_ui(ui, |ui| {
                for named_style in EaseStyle::iter() {
                    let n = named_style.to_string();
                    ui.selectable_value(style, named_style, n);
                }
            });

        let PhaseSettings { duration, style } = release_settings;

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Release duration:");
            ui.add(egui::Slider::new(duration, 1..=10000).suffix("ms"));
        });

        ComboBox::from_label("Release-phase Tween")
            .selected_text(style.to_string())
            .show_ui(ui, |ui| {
                for named_style in EaseStyle::iter() {
                    let n = named_style.to_string();
                    ui.selectable_value(style, named_style, n);
                }
            });

        ui.separator();

        let TransmissionSettings {
            max_range,
            max_delay,
        } = transmission_settings;

        ui.horizontal(|ui| {
            ui.label("Transmission range");
            ui.add(egui::Slider::new(max_range, 0. ..=1000.).suffix("px"));
        });

        ui.horizontal(|ui| {
            ui.label("Transmission max delay");
            ui.add(egui::Slider::new(max_delay, 0..=4000).suffix("ms"))
        });
    });
}
