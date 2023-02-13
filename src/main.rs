use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};

const COUNT: usize = 7;
const THICKNESS: f32 = 10.;
const LENGTH: f32 = 200.;
const ATTACK_DURATION: u16 = 125;
const RELEASE_DURATION: u16 = 2500;

mod animation;
use crate::animation::*;

mod particles;
use crate::particles::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Settings {
    chimes_count: usize,
    chime_thickness: f32,
    chime_length: f32,
    attack_duration: u16,
    release_duration: u16,
}
struct Model {
    window_id: WindowId,
    particles: Vec<Particle>,
    mouse_position: Point2,
    egui: Egui,
    settings: Settings,
}

// ---------------- Event Handlers

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    println!("mouse pressed at position {}", model.mouse_position);
    if let Some(close_particle) = model.particles.iter_mut().find(|p| {
        let distance = p.position.distance(model.mouse_position);
        // println!("distance: {}", distance);
        distance <= LENGTH / 2.
    }) {
        println!("clicked on particle!");
        close_particle.animation = EnvelopeStage::AttackAnimation(Attack::new(
            model.settings.attack_duration,
            close_particle.brightness,
            app.duration.since_start.as_millis(),
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
        particles: build_layout(COUNT, window.rect().w() * 0.8, window.rect().h() * 0.2),
        settings: Settings {
            chimes_count: COUNT,
            chime_thickness: THICKNESS,
            chime_length: LENGTH,
            attack_duration: ATTACK_DURATION,
            release_duration: RELEASE_DURATION,
        },
        egui,
    }
}

// ---------------- Update before drawing ever frame

fn update(app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    egui::Window::new("Settings").show(&ctx, |ui| {
        let settings = &mut model.settings;

        ui.label("Chimes count:");
        ui.add(egui::Slider::new(&mut settings.chimes_count, 1..=30));
        let current_count = settings.chimes_count.to_owned();
        if ui.button("update").clicked() {
            // println!("chimes count change to {}", model.settings.chimes_count);
            let window = app.window(model.window_id).unwrap();

            model.particles = build_layout(
                current_count,
                window.rect().w() * 0.8,
                window.rect().h() * 0.2,
            )
        }

        // TODO: below cannot work because of ownership
        // let window = app.window(model.window_id).unwrap();
        // if response.changed() {
        //     println!("chimes count change to {}", model.settings.chimes_count);
        //     // model.particles = build_layout(
        //     //     model.settings.chimes_count,
        //     //     Point2::new(window.rect().left(), 0.),
        //     //     window.rect().w() / COUNT.to_f32().unwrap(),
        //     // )
        // }

        ui.separator();

        ui.label("Chimes thickness:");
        ui.add(egui::Slider::new(&mut settings.chime_thickness, 1. ..=200.));

        ui.label("Chimes length:");
        ui.add(egui::Slider::new(&mut settings.chime_length, 10. ..=2000.));

        ui.separator();

        ui.label("Attack duration:");
        ui.add(egui::Slider::new(&mut settings.attack_duration, 1..=2000));

        ui.label("Release duration:");
        ui.add(egui::Slider::new(&mut settings.release_duration, 1..=15000));
    });

    for p in &mut model.particles {
        let animation = &mut p.animation;
        let current_time = app.duration.since_start.as_millis();

        match animation {
            EnvelopeStage::AttackAnimation(a) => {
                let (brightness, done) = a.get_brightness_and_done(current_time);
                p.brightness = brightness;
                if done {
                    println!("end Attack => Release");
                    p.animation = EnvelopeStage::ReleaseAnimation(Release::new(
                        model.settings.release_duration,
                        p.brightness,
                        current_time,
                    ))
                }
            }
            EnvelopeStage::ReleaseAnimation(a) => {
                let (brightness, done) = a.get_brightness_and_done(current_time);
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
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
