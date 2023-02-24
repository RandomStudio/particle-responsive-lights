use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, info, warn};
use nannou::prelude::*;
use nannou_egui::Egui;
use settings::{build_ui, Cli, EaseStyle, PhaseSettings, DEFAULT_WINDOW_H, DEFAULT_WINDOW_W};
use settings::{Model, TransmissionSettings};

mod animation;
use crate::animation::*;

mod settings;

mod particles;
use crate::particles::*;
use crate::settings::get_tween;

mod artnet;

mod tether;

fn main() {
    nannou::app(model).update(update).run();
}

// ---------------- Event Handlers

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    debug!("mouse pressed at position {}", model.mouse_position);
    if !model.settings.mouse_enable {
        warn!("mouse click ignored; mouse control disabled");
        return;
    }
    let PhaseSettings { style, .. } = &model.settings.attack_settings;
    let attack_duration = model.settings.attack_settings.duration;
    let release_duration = model.settings.release_settings.duration;
    let TransmissionSettings {
        max_range,
        max_delay,
    } = &model.settings.transmission_settings;

    let particles = &mut model.particles;

    if let Some(target_particle) = particles.iter().find(|p| {
        let tolerance = &model.settings.chime_thickness * 2.;
        let left = p.position.x - tolerance;
        let right = p.position.x + tolerance;
        model.mouse_position.x >= left && model.mouse_position.x <= right
    }) {
        let id = target_particle.id;
        let position = target_particle.position;
        let max_range_pixels = *max_range * DEFAULT_WINDOW_W.to_f32().unwrap();
        trigger_activation(
            particles,
            id,
            position,
            model.settings.mouse_brightness_value,
            model.settings.resting_brightness,
            attack_duration,
            release_duration,
            max_range_pixels,
            *max_delay,
            style,
        );
    }
}

fn trigger_activation(
    particles: &mut Vec<Particle>,
    main_target_id: usize,
    main_target_position: Point2,
    brightness: f32,
    final_brightness: f32,
    attack_duration: usize,
    release_duration: usize,
    max_range: f32,
    max_delay: i64,
    style: &EaseStyle,
) {
    for p in particles {
        if p.id == main_target_id {
            activate_single(
                p,
                attack_duration,
                release_duration,
                style,
                p.brightness(),
                brightness,
                final_brightness,
                0,
            );
        } else {
            // let distance = main_target_position.distance(p.position);
            let distance = (main_target_position.x - p.position.x).abs();
            if distance <= max_range {
                if let Some(new_brightness_target) =
                    possibly_activate_by_transmission(p, distance, max_range, brightness)
                {
                    activate_single(
                        p,
                        attack_duration,
                        release_duration,
                        style,
                        p.brightness(),
                        new_brightness_target,
                        final_brightness,
                        map_range(distance, 0., max_range, 0, max_delay),
                    )
                }
            }
        }
    }
}

fn activate_single(
    p: &mut Particle,
    attack_duration: usize,
    release_duration: usize,
    ease_style: &EaseStyle,
    start_brightness: f32,
    target_brightness: f32,
    final_brightness: f32,
    delay: i64,
) {
    let mut attack = Attack::new(
        attack_duration,
        start_brightness,
        target_brightness,
        get_tween(ease_style),
    );
    attack.set_elapsed(-delay);
    p.animation = EnvelopeStage::AttackAnimation(
        attack,
        Some(AfterAttack {
            release_duration,
            final_brightness,
        }),
    );
    debug!(
        "#{} activate to target_brightness {}",
        p.id, target_brightness
    );
}

fn possibly_activate_by_transmission(
    p: &mut Particle,
    distance: f32,
    max_range: f32,
    feed_in_brightness: f32,
) -> Option<f32> {
    let current_brightness = p.brightness();
    let target_brightness = map_range(distance, 0., max_range, 1., 0.0) * feed_in_brightness;
    if target_brightness > current_brightness {
        Some(target_brightness)
    } else {
        None
    }
}

fn mouse_moved(_app: &App, model: &mut Model, pos: Point2) {
    model.mouse_position = pos;
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

// ---------------- Set up Model with defaults, some overridden by command-line args

fn model(app: &App) -> Model {
    let cli = Cli::parse();

    // Initialize the logger from the environment
    // env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level)).init();
    // env_logger::init();

    let mut builder = Builder::from_env(Env::default().default_filter_or(&cli.log_level));
    builder.filter_module("wgpu_core", log::LevelFilter::Error);
    builder.filter_module("wgpu_hal", log::LevelFilter::Warn);
    builder.filter_module("naga", log::LevelFilter::Warn);
    builder.init();
    info!("Started; args: {:?}", cli);
    debug!("Debugging is enabled; could be verbose");

    let window_id = app
        .new_window()
        .size(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    Model::defaults(window_id, egui, &cli)
}

// ---------------- Update before drawing every frame

fn update(app: &App, model: &mut Model, update: Update) {
    let window = app.window(model.window_id).unwrap();

    build_ui(model, update.since_start, window.rect());

    let delta_time = app
        .duration
        .since_prev_update
        .as_millis()
        .to_usize()
        .unwrap();

    let title = format!(
        "Particle Lights @{:?}fps",
        (1000. / delta_time.to_f32().unwrap()).to_u32().unwrap_or(0)
    );
    app.main_window().set_title(&title);

    for p in &mut model.particles {
        let animation = &mut p.animation;
        // let current_time = app.duration.since_start.as_millis();

        match animation {
            EnvelopeStage::AttackAnimation(a, after_release) => {
                let (brightness, done) = a.get_brightness_and_done(delta_time);
                // p.set_brightness(brightness);
                if done {
                    debug!("#{} end Attack => Release", p.id);
                    let (duration, final_brightness) = {
                        if let Some(after) = after_release {
                            (after.release_duration, after.final_brightness)
                        } else {
                            (model.settings.release_settings.duration, 0.)
                        }
                    };
                    p.animation = EnvelopeStage::ReleaseAnimation(Release::new(
                        duration,
                        p.brightness(),
                        final_brightness,
                        get_tween(&model.settings.release_settings.style),
                    ))
                } else {
                    p.set_brightness(brightness);
                }
            }
            EnvelopeStage::ReleaseAnimation(a) => {
                let (brightness, done) = a.get_brightness_and_done(delta_time);
                p.set_brightness(brightness);
                if done {
                    debug!("#{} end Release => Idle", p.id);
                    p.animation = EnvelopeStage::Idle()
                }
            }
            EnvelopeStage::Idle() => {}
        }
    }

    model.artnet.update(&model.particles);

    if model.tether.is_connected() {
        let PhaseSettings { style, .. } = &model.settings.attack_settings;
        let TransmissionSettings {
            max_range,
            max_delay,
        } = &model.settings.transmission_settings;
        let trigger_by_order = model.settings.trigger_by_order;
        let particles = &mut model.particles;
        if let Some(light_message) = model.tether.check_messages() {
            if let Some(target_particle) = particles.iter().find(|p| {
                light_message.id == {
                    if trigger_by_order {
                        p.order
                    } else {
                        p.id
                    }
                }
            }) {
                let position = target_particle.position;
                let id = target_particle.id;
                let trigger_brightness = {
                    if model.settings.trigger_full_brightness {
                        1.
                    } else {
                        light_message.target_brightness
                    }
                };
                let max_range_pixels = *max_range * DEFAULT_WINDOW_W.to_f32().unwrap();

                let attack_duration = {
                    if light_message.attack_duration > 0 {
                        light_message.attack_duration
                    } else {
                        model.settings.attack_settings.duration
                    }
                };

                let release_duration = {
                    if light_message.release_duration > 0 {
                        light_message.release_duration
                    } else {
                        model.settings.release_settings.duration
                    }
                };

                let final_brightness = {
                    if light_message.final_brightness > 0. {
                        light_message.final_brightness
                    } else {
                        model.settings.resting_brightness
                    }
                };

                trigger_activation(
                    particles,
                    id,
                    position,
                    trigger_brightness,
                    final_brightness,
                    attack_duration,
                    release_duration,
                    max_range_pixels,
                    *max_delay,
                    style,
                );
            }
        }
    }
}

// ---------------- Draw every frame

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(DARKSLATEGREY);

    for p in &model.particles {
        draw
            // .ellipse()
            .rect()
            .w_h(model.settings.chime_thickness, model.settings.chime_length)
            .x_y(p.position.x, p.position.y)
            .color(gray(p.brightness()));

        if model.settings.show_brightness_indicator {
            let size = model.settings.chime_length / 2.;
            draw.rect()
                .w_h(model.settings.chime_thickness * 1.25, 2.)
                .x_y(
                    p.position.x,
                    p.position.y + map_range(p.brightness(), 0., 1., size, -size),
                )
                .color({
                    match p.animation {
                        EnvelopeStage::AttackAnimation(_, _) => GREEN,
                        EnvelopeStage::ReleaseAnimation(_) => ORANGERED,
                        EnvelopeStage::Idle() => WHITE,
                    }
                });
        }
        if model.settings.show_chime_index {
            let size = model.settings.chime_length / 2.;
            let text: &str = &format!("#{} ({})", p.id, p.order);
            draw.text(text)
                .color(SLATEGREY)
                .x_y(p.position.x, p.position.y + size * 1.1);
        }
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
