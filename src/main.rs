use nannou::prelude::*;

const THICKNESS: f32 = 10.;
const LENGTH: f32 = 200.;
const ATTACK_DURATION: u128 = 125;
const RELEASE_DURATION: u128 = 2500;

pub trait Animation {
    fn new(duration: u128, start_brightness: f32, start_time: u128) -> Self;
    fn duration(&self) -> u128;
    fn start_time(&self) -> u128;

    fn get_range(&self) -> (f32, f32);

    /// Update the animation with the current time, get the progress in the range `[0,1]`
    fn update(&self, current_time: u128) -> f32 {
        let elapsed = current_time - self.start_time();
        let progress = elapsed.to_f64().unwrap() / self.duration().to_f64().unwrap();
        progress.to_f32().unwrap()
    }

    fn get_brightness_and_done(&self, current_time: u128) -> (f32, bool) {
        let (from, to) = self.get_range();
        let progress = self.update(current_time);
        let brightness = map_range(progress, 0., 1., from, to);
        if progress > 1. {
            (to, true)
        } else {
            (brightness, false)
        }
    }
}

struct Particle {
    position: Point2,
    brightness: f32,
    animation: EnvelopeStage,
}

struct Attack {
    start_time: u128,
    duration: u128,
    range: (f32, f32),
}

impl Animation for Attack {
    fn new(duration: u128, start_brightness: f32, start_time: u128) -> Self {
        Attack {
            start_time,
            duration,
            range: (start_brightness, 1.0),
        }
    }

    fn start_time(&self) -> u128 {
        self.start_time
    }
    fn duration(&self) -> u128 {
        self.duration
    }
    fn get_range(&self) -> (f32, f32) {
        self.range
    }
}
struct Release {
    start_time: u128,
    duration: u128,
    range: (f32, f32),
}

impl Animation for Release {
    fn new(duration: u128, start_brightness: f32, start_time: u128) -> Self {
        Release {
            start_time,
            duration,
            range: (start_brightness, 0.0),
        }
    }
    fn start_time(&self) -> u128 {
        self.start_time
    }
    fn duration(&self) -> u128 {
        self.duration
    }
    fn get_range(&self) -> (f32, f32) {
        self.range
    }
}

// The animation concept is based on https://en.wikipedia.org/wiki/Envelope_(music)
enum EnvelopeStage {
    // Active(Box<dyn Animated>),
    AttackAnimation(Attack),
    ReleaseAnimation(Release),
    Idle(),
}

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    particles: Vec<Particle>,
    mouse_position: Point2,
}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    println!("mouse pressed at position {}", model.mouse_position);
    if let Some(close_particle) = model.particles.iter_mut().find(|p| {
        let distance = p.position.distance(model.mouse_position);
        // println!("distance: {}", distance);
        distance <= LENGTH / 2.
    }) {
        println!("clicked on particle!");
        close_particle.animation = EnvelopeStage::AttackAnimation(Attack::new(
            ATTACK_DURATION,
            close_particle.brightness,
            app.duration.since_start.as_millis(),
        ))
    }
}

fn mouse_moved(_app: &App, model: &mut Model, pos: Point2) {
    model.mouse_position = pos;
}

fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .build()
        .unwrap();
    Model {
        _window,
        mouse_position: Point2::new(0., 0.),
        particles: vec![
            Particle {
                position: Vec2::new(0., 0.),
                brightness: 0.,
                animation: EnvelopeStage::Idle(),
            },
            Particle {
                position: Vec2::new(100., 100.),
                brightness: 0.,
                animation: EnvelopeStage::Idle(),
            },
            Particle {
                position: Vec2::new(200., 25.),
                brightness: 0.,
                animation: EnvelopeStage::Idle(),
            },
            Particle {
                position: Vec2::new(-100., 50.),
                brightness: 0.,
                animation: EnvelopeStage::Idle(),
            },
        ],
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
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
                        RELEASE_DURATION,
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
            .w_h(THICKNESS, LENGTH)
            .x_y(p.position.x, p.position.y)
            .color(gray(p.brightness));
    }

    draw.to_frame(app, &frame).unwrap();
}
