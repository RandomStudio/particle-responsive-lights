use nannou::prelude::*;
use nannou_egui::Egui;
use settings::{build_ui, EaseStyle, PhaseSettings, DEFAULT_WINDOW_H, DEFAULT_WINDOW_W};
use settings::{Model, TransmissionSettings};

mod animation;
use crate::animation::*;

mod settings;

mod particles;
use crate::particles::*;
use crate::settings::get_tween;
use crate::tether::TetherConnection;

mod artnet;

mod tether;

fn main() {
    println!("Started");

    nannou::app(model).update(update).run();
}

// ---------------- Event Handlers

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    println!("mouse pressed at position {}", model.mouse_position);
    let PhaseSettings { duration, style } = &model.settings.attack_settings;
    let TransmissionSettings {
        max_range,
        max_delay,
    } = &model.settings.transmission_settings;

    if let Some(target_particle) = model.particles.iter().find(|p| {
        let tolerance = &model.settings.chime_thickness * 2.;
        let left = p.position.x - tolerance;
        let right = p.position.x + tolerance;
        model.mouse_position.x >= left && model.mouse_position.x <= right
    }) {
        let id = target_particle.id;
        let position = target_particle.position;
        for p in model.particles.iter_mut() {
            if p.id == id {
                activate_single(p, *duration, style, p.brightness, 1.0, 0);
            } else {
                let distance = position.distance(p.position);
                if distance <= *max_range {
                    if let Some(new_brightness_target) =
                        possibly_activate_by_transmission(p, distance, *max_range)
                    {
                        activate_single(
                            p,
                            *duration,
                            style,
                            p.brightness,
                            new_brightness_target,
                            map_range(distance, 0., *max_range, 0, *max_delay),
                        )
                    }
                }
            }
        }
    }
}

fn activate_single(
    p: &mut Particle,
    duration: usize,
    ease_style: &EaseStyle,
    start_brightness: f32,
    target_brightness: f32,
    delay: i64,
) {
    let mut attack = Attack::new(
        duration,
        start_brightness,
        target_brightness,
        get_tween(ease_style),
    );
    attack.set_elapsed(-delay);
    p.animation = EnvelopeStage::AttackAnimation(attack);
    println!("#{} activate", p.id);
}

fn possibly_activate_by_transmission(
    p: &mut Particle,
    distance: f32,
    max_range: f32,
) -> Option<f32> {
    let current_brightness = p.brightness;
    let target_brightness = map_range(distance, 0., max_range, 1., 0.0);
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

// ---------------- Set up Model with defaults

fn model(app: &App) -> Model {
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

    let mut tether = TetherConnection::new();
    tether.connect();
    Model::defaults(window_id, egui, tether)
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
            EnvelopeStage::AttackAnimation(a) => {
                let (brightness, done) = a.get_brightness_and_done(delta_time);
                p.brightness = brightness;
                if done {
                    println!("#{} end Attack => Release", p.id);
                    p.animation = EnvelopeStage::ReleaseAnimation(Release::new(
                        model.settings.release_settings.duration,
                        p.brightness,
                        0.,
                        get_tween(&model.settings.release_settings.style),
                    ))
                }
            }
            EnvelopeStage::ReleaseAnimation(a) => {
                let (brightness, done) = a.get_brightness_and_done(delta_time);
                p.brightness = brightness;
                if done {
                    println!("#{} end Release => Idle", p.id);
                    p.animation = EnvelopeStage::Idle()
                }
            }
            EnvelopeStage::Idle() => {}
        }
    }

    model.artnet.update(&model.particles);

    if model.tether.is_connected() {
        model.tether.check_messages();
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
            .color(gray(p.brightness));

        if model.settings.show_brightness_indicator {
            let size = model.settings.chime_length / 2.;
            draw.rect()
                .w_h(model.settings.chime_thickness * 1.25, 2.)
                .x_y(
                    p.position.x,
                    p.position.y + map_range(p.brightness, 0., 1., size, -size),
                )
                .color(WHITE);
        }
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
