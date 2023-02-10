use nannou::prelude::*;

pub trait Animated {
    fn duration(&self) -> u128;
    fn start_time(&self) -> u128;

    /// Update the animation with the current time, get the progress in the range `[0,1]`
    fn update(&self, current_time: u128) -> f32 {
        let elapsed = current_time - self.start_time();
        let progress = elapsed.to_f64().unwrap() / self.duration().to_f64().unwrap();
        progress.to_f32().unwrap()
    }
}

#[derive(Debug)]
struct Particle {
    position: Point2,
    brightness: f32,
    animation: EnvelopeStage,
}

#[derive(Debug)]
struct Attack {
    start_time: u128,
    duration: u128,
}

impl Animated for Attack {
    fn start_time(&self) -> u128 {
        self.start_time
    }
    fn duration(&self) -> u128 {
        self.duration
    }
}
#[derive(Debug)]
struct Release {
    start_time: u128,
    duration: u128,
}

impl Animated for Release {
    fn start_time(&self) -> u128 {
        self.start_time
    }
    fn duration(&self) -> u128 {
        self.duration
    }
}

// The animation concept is based on https://en.wikipedia.org/wiki/Envelope_(music)
#[derive(Debug)]
enum EnvelopeStage {
    AttackAnimation(Attack),
    ReleaseAnimation(Release),
    Idle(),
}

const THICKNESS: f32 = 5.;
const LENGTH: f32 = 100.;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    particles: Vec<Particle>,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    Model {
        _window,
        particles: vec![
            Particle {
                position: Vec2::new(0., 0.),
                brightness: 1.0,
                animation: EnvelopeStage::Idle(),
            },
            Particle {
                position: Vec2::new(100., 100.),
                brightness: 0.5,
                animation: EnvelopeStage::Idle(),
            },
            Particle {
                position: Vec2::new(-100., 50.),
                brightness: 0.,
                animation: EnvelopeStage::AttackAnimation(Attack {
                    start_time: app.duration.since_prev_update.as_millis(),
                    duration: 5000,
                }),
            },
        ],
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    for p in &mut model.particles {
        let animation = &mut p.animation;
        // let current_time = app.duration.since_prev_update.as_millis();
        let current_time = app.duration.since_start.as_millis();
        println!("current time {}", current_time);
        if let Some(progress) = match animation {
            EnvelopeStage::AttackAnimation(a) => Some(a.update(current_time)),
            EnvelopeStage::ReleaseAnimation(a) => Some(a.update(current_time)),
            EnvelopeStage::Idle() => None,
        } {
            println!("updating {:?} with progress {}", p, progress);
            p.brightness = if progress > 1. { 1. } else { progress };
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
