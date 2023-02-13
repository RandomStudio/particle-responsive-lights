use nannou::prelude::*;
use nannou_egui::Egui;
use settings::{build_ui, EaseStyle, PhaseSettings, Settings};

const DEFAULT_COUNT: usize = 7;
const DEFAULT_THICKNESS: f32 = 10.;
const DEFAULT_LENGTH: f32 = 200.;
const DEFAULT_ATTACK_DURATION: usize = 300;
const DEFAULT_RELEASE_DURATION: usize = 2500;
const DEFAULT_SHOW_B_INDICATOR: bool = true;
pub const DEFAULT_WIDTH_RATIO: f32 = 0.6;

mod animation;
use crate::animation::*;

mod settings;

mod particles;
use crate::particles::*;
use crate::settings::get_tween;

fn main() {
    nannou::app(model).update(update).run();
}

pub struct Model {
    window_id: WindowId,
    particles: Vec<Particle>,
    mouse_position: Point2,
    egui: Egui,
    settings: Settings,
}

// ---------------- Event Handlers

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    println!("mouse pressed at position {}", model.mouse_position);
    if let Some(close_particle) = model.particles.iter_mut().find(|p| {
        let distance = p.position.distance(model.mouse_position);
        // println!("distance: {}", distance);
        distance <= DEFAULT_LENGTH / 2.
    }) {
        println!("clicked on particle!");
        close_particle.animation = EnvelopeStage::AttackAnimation(Attack::new(
            model.settings.attack_settings.duration,
            close_particle.brightness,
            get_tween(&model.settings.attack_settings.style),
        ))
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
        .size(1920, 720)
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    Model {
        window_id,
        mouse_position: Point2::new(0., 0.),
        particles: build_layout(
            DEFAULT_COUNT,
            window.rect().w() * DEFAULT_WIDTH_RATIO,
            window.rect().h() * 0.2,
        ),
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
            show_brightness_indicator: DEFAULT_SHOW_B_INDICATOR,
        },
        egui,
    }
}

// ---------------- Update before drawing ever frame

fn update(app: &App, model: &mut Model, update: Update) {
    let window = app.window(model.window_id).unwrap();

    build_ui(model, update.since_start, window.rect());

    for p in &mut model.particles {
        let animation = &mut p.animation;
        // let current_time = app.duration.since_start.as_millis();
        let delta_time = app
            .duration
            .since_prev_update
            .as_millis()
            .to_usize()
            .unwrap();

        match animation {
            EnvelopeStage::AttackAnimation(a) => {
                let (brightness, done) = a.get_brightness_and_done(delta_time);
                p.brightness = brightness;
                if done {
                    println!("end Attack => Release");
                    p.animation = EnvelopeStage::ReleaseAnimation(Release::new(
                        model.settings.release_settings.duration,
                        p.brightness,
                        get_tween(&model.settings.release_settings.style),
                    ))
                }
            }
            EnvelopeStage::ReleaseAnimation(a) => {
                let (brightness, done) = a.get_brightness_and_done(delta_time);
                p.brightness = brightness;
                if done {
                    println!("end Release => Idle");
                    p.animation = EnvelopeStage::Idle()
                }
            }
            EnvelopeStage::Idle() => {}
        }
    }
}

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
