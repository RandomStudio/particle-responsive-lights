use std::time::Duration;

use nannou::prelude::*;
use nannou_egui::egui::{self, ComboBox, Slider};

use strum::IntoEnumIterator;

use crate::particles::build_layout;
use crate::settings::{
    EaseStyle, Model, PhaseSettings, Settings, TransmissionSettings, DEFAULT_WIDTH_RATIO,
};

pub fn build_ui(model: &mut Model, since_start: Duration, window_rect: Rect) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        let Settings {
            chimes_count,
            show_brightness_indicator,
            chime_thickness,
            chime_length,
            attack_settings,
            release_settings,
            transmission_settings,
            trigger_full_brightness,
            trigger_by_order,
            mouse_enable,
            mouse_brightness_value,
            resting_brightness,
            ..
        } = &mut model.settings;

        ui.set_min_height(600.);

        ui.collapsing("View / Interaction", |ui| {
            ui.horizontal(|ui| {
                ui.label("Chimes count:");
                ui.add(Slider::new(chimes_count, 1..=30));
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
            ui.checkbox(mouse_enable, "Allow mouse click triggers");

            if *mouse_enable {
                ui.horizontal(|ui| {
                    ui.label("Mouse click strength:");
                    ui.add(Slider::new(mouse_brightness_value, 0. ..=1.).suffix("x"));
                });
            }

            ui.horizontal(|ui| {
                ui.label("Chimes thickness:");
                ui.add(Slider::new(chime_thickness, 1. ..=200.).suffix("px"));
            });

            ui.horizontal(|ui| {
                ui.label("Chimes length:");
                ui.add(Slider::new(chime_length, 10. ..=2000.).suffix("px"));
            });
        });

        ui.collapsing("Animation", |ui| {
            let PhaseSettings { duration, style } = attack_settings;

            ui.horizontal(|ui| {
                ui.label("Attack duration:");
                ui.add(Slider::new(duration, 1..=10000).suffix("ms"));
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
                ui.add(Slider::new(duration, 1..=10000).suffix("ms"));
            });

            ComboBox::from_label("Release-phase Tween")
                .selected_text(style.to_string())
                .show_ui(ui, |ui| {
                    for named_style in EaseStyle::iter() {
                        let n = named_style.to_string();
                        ui.selectable_value(style, named_style, n);
                    }
                });

            ui.horizontal(|ui| {
                ui.label("Rest brightness after release:");
                ui.add(Slider::new(resting_brightness, 0. ..=1.).suffix("x"));
            });

            ui.separator();

            let TransmissionSettings {
                max_range,
                max_delay,
            } = transmission_settings;

            ui.horizontal(|ui| {
                ui.label("Transmission range factor");
                ui.add(Slider::new(max_range, 0. ..=1.).suffix("x"));
            });

            ui.horizontal(|ui| {
                ui.label("Transmission max delay");
                ui.add(Slider::new(max_delay, 0..=4000).suffix("ms"))
            });
        });
        ui.collapsing("Remote Control", |ui| {
            ui.checkbox(
                trigger_full_brightness,
                "Remote trigger max brightness always",
            );
            ui.checkbox(trigger_by_order, "Remote trigger by order not #ID");
        });
    });
}