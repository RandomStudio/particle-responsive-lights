use nannou::prelude::{Point2, ToPrimitive};

use crate::animation::EnvelopeStage;

pub struct Particle {
    pub position: Point2,
    pub brightness: f32,
    pub animation: EnvelopeStage,
}

impl Particle {
    fn new(position: Point2) -> Self {
        Particle {
            position,
            brightness: 0.,
            animation: EnvelopeStage::Idle(),
        }
    }
}

pub fn build_layout(count: usize, start_position: Point2, gap: f32) -> Vec<Particle> {
    let mut particles: Vec<Particle> = vec![];
    for i in 0..count {
        particles.push(Particle::new(Point2::new(
            start_position.x + gap / 2. + i.to_f32().unwrap() * gap,
            i.to_f32().unwrap().sin() * gap,
        )))
    }
    particles
}
