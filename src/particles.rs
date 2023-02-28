use nannou::prelude::{map_range, Point2, ToPrimitive};

use crate::{animation::EnvelopeStage, settings::DEFAULT_ORDER};

pub struct Particle {
    pub id: usize,
    pub order: usize,
    pub position: Point2,
    brightness: f32,
    pub animation: EnvelopeStage,
}

impl Particle {
    fn new(id: usize, order: usize, position: Point2) -> Self {
        Particle {
            id,
            order,
            position,
            brightness: 0.,
            animation: EnvelopeStage::Idle(),
        }
    }
    pub fn brightness(&self) -> f32 {
        self.brightness.clamp(0., 1.0)
    }
    pub fn set_brightness(&mut self, new_value: f32) {
        self.brightness = new_value;
    }
}

pub fn build_layout(count: usize, width_range: f32, height_range: f32) -> Vec<Particle> {
    let gap_x = width_range / count.to_f32().unwrap();
    let start_position = Point2::new(-width_range / 2. + gap_x / 2., -height_range / 2.);
    let mut particles: Vec<Particle> = vec![];
    for (i, id) in DEFAULT_ORDER.iter().enumerate().take(count) {
        println!("assign order {id} to ID #{i}");
        particles.push(Particle::new(
            *id,
            i,
            Point2::new(
                start_position.x + gap_x * i.to_f32().unwrap(),
                map_range(
                    i.to_f32().unwrap().sin(),
                    -1.,
                    1.,
                    -height_range / 2.,
                    height_range / 2.,
                ),
            ),
        ))
    }
    particles.sort_by_key(|p| p.id);
    particles
}
