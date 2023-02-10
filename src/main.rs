use nannou::prelude::*;
struct Particle {
    position: Point2,
    brightness: f32
}

const THICKNESS: f32 = 5.;
const LENGTH: f32 = 100.;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    _window: window::Id,
    particles: Vec<Particle>
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();
    Model { _window, particles: vec![
        Particle{
            position: Vec2::new(0., 0.),
            brightness: 1.0
        },
        Particle{
            position: Vec2::new(100., 100.),
            brightness: 0.5
        },
        Particle{
            position: Vec2::new(-100., 50.),
            brightness: 0.5
        }

    ] }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

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

