use std::time::Duration;

use nannou::prelude::Rect;
use nannou_egui::egui::{self, ComboBox};
use tween::*;

use crate::{particles::build_layout, Model};

use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;

pub struct PhaseSettings {
    pub duration: usize,
    pub style: EaseStyle,
}

pub struct Settings {
    pub chimes_count: usize,
    pub show_brightness_indicator: bool,
    pub chime_thickness: f32,
    pub chime_length: f32,
    pub attack_settings: PhaseSettings,
    pub release_settings: PhaseSettings,
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
            ..
        } = &mut model.settings;

        ui.set_min_height(600.);

        ui.label("Chimes count:");
        ui.add(egui::Slider::new(chimes_count, 1..=30));
        let current_count = chimes_count.to_owned();
        if ui.button("update").clicked() {
            model.particles =
                build_layout(current_count, window_rect.w() * 0.8, window_rect.h() * 0.2)
        }

        ui.separator();

        ui.checkbox(show_brightness_indicator, "Brightness indicator");

        ui.label("Chimes thickness:");
        ui.add(egui::Slider::new(chime_thickness, 1. ..=200.));

        ui.label("Chimes length:");
        ui.add(egui::Slider::new(chime_length, 10. ..=2000.));

        ui.separator();

        let PhaseSettings { duration, style } = attack_settings;

        ui.label("Attack duration:");
        ui.add(egui::Slider::new(duration, 1..=10000));
        ui.add(egui::DragValue::new(duration).clamp_range(1..=10000));

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

        ui.label("Release duration:");

        ui.add(egui::Slider::new(duration, 1..=10000));
        ui.add(egui::DragValue::new(duration).clamp_range(1..=10000));

        ComboBox::from_label("Release-phase Tween")
            .selected_text(style.to_string())
            .show_ui(ui, |ui| {
                for named_style in EaseStyle::iter() {
                    let n = named_style.to_string();
                    ui.selectable_value(style, named_style, n);
                }
            });
    });
}
