use nannou::prelude::*;
use nannou_egui::egui::ComboBox;
use nannou_egui::{self, egui, Egui};
use tween::{BounceIn, BounceOut, Linear, SineInOut, Tween};

const DEFAULT_COUNT: usize = 7;
const DEFAULT_THICKNESS: f32 = 10.;
const DEFAULT_LENGTH: f32 = 200.;
const DEFAULT_ATTACK_DURATION: usize = 125;
const DEFAULT_RELEASE_DURATION: usize = 2500;
const DEFAULT_SHOW_B_INDICATOR: bool = true;

mod animation;
use crate::animation::*;

mod particles;
use crate::particles::*;

fn main() {
    nannou::app(model).update(update).run();
}

#[derive(PartialEq, Debug)]
enum EaseStyle {
    Linear,
    BounceIn,
    BounceOut,
    SineInOut,
}

fn get_tween(style: &EaseStyle) -> Box<dyn Tween<f32>> {
    match style {
        EaseStyle::Linear => Box::new(Linear),
        EaseStyle::BounceIn => Box::new(BounceIn),
        EaseStyle::BounceOut => Box::new(BounceOut),
        EaseStyle::SineInOut => Box::new(SineInOut),
    }
}

struct PhaseSettings {
    duration: usize,
    style: EaseStyle,
}

struct Settings {
    chimes_count: usize,
    show_brightness_indicator: bool,
    chime_thickness: f32,
    chime_length: f32,
    attack_settings: PhaseSettings,
    release_settings: PhaseSettings,
}
struct Model {
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
            window.rect().w() * 0.8,
            window.rect().h() * 0.2,
        ),
        settings: Settings {
            chimes_count: DEFAULT_COUNT,
            chime_thickness: DEFAULT_THICKNESS,
            chime_length: DEFAULT_LENGTH,
            attack_settings: PhaseSettings {
                duration: DEFAULT_ATTACK_DURATION,
                style: EaseStyle::SineInOut,
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

        ui.separator();

        ui.checkbox(
            &mut settings.show_brightness_indicator,
            "Brightness indicator",
        );

        ui.label("Chimes thickness:");
        ui.add(egui::Slider::new(&mut settings.chime_thickness, 1. ..=200.));

        ui.label("Chimes length:");
        ui.add(egui::Slider::new(&mut settings.chime_length, 10. ..=2000.));

        ui.separator();

        ui.label("Attack duration:");
        ui.add(egui::Slider::new(
            &mut settings.attack_settings.duration,
            1..=10000,
        ));
        ui.add(egui::DragValue::new(&mut settings.attack_settings.duration).clamp_range(1..=10000));

        ui.label("Release duration:");

        ui.add(egui::Slider::new(
            &mut settings.release_settings.duration,
            1..=10000,
        ));
        ui.add(
            egui::DragValue::new(&mut settings.release_settings.duration).clamp_range(1..=10000),
        );

        ui.separator();

        ComboBox::from_label("Attack-phase Tween")
            .selected_text(format!("{:?}", model.settings.attack_settings.style))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                ui.selectable_value(
                    &mut model.settings.attack_settings.style,
                    EaseStyle::Linear,
                    "Linear",
                );
                ui.selectable_value(
                    &mut model.settings.attack_settings.style,
                    EaseStyle::BounceIn,
                    "BounceIn",
                );
                ui.selectable_value(
                    &mut model.settings.attack_settings.style,
                    EaseStyle::BounceOut,
                    "BounceOut",
                );
                ui.selectable_value(
                    &mut model.settings.attack_settings.style,
                    EaseStyle::SineInOut,
                    "SineInOut",
                );
            });

        ComboBox::from_label("Release-phase Tween")
            .selected_text(format!("{:?}", model.settings.release_settings.style))
            .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                ui.selectable_value(
                    &mut model.settings.release_settings.style,
                    EaseStyle::Linear,
                    "Linear",
                );
                ui.selectable_value(
                    &mut model.settings.release_settings.style,
                    EaseStyle::BounceIn,
                    "BounceIn",
                );
                ui.selectable_value(
                    &mut model.settings.release_settings.style,
                    EaseStyle::BounceOut,
                    "BounceOut",
                );
                ui.selectable_value(
                    &mut model.settings.release_settings.style,
                    EaseStyle::SineInOut,
                    "SineInOut",
                );
            });
    });

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
