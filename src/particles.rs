use nannou::prelude::{map_range, Point2, ToPrimitive};

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

pub fn build_layout(count: usize, width_range: f32, height_range: f32) -> Vec<Particle> {
    let gap_x = width_range / count.to_f32().unwrap();
    let start_position = Point2::new(-width_range / 2. + gap_x / 2., -height_range / 2.);
    let mut particles: Vec<Particle> = vec![];
    for i in 0..count {
        particles.push(Particle::new(Point2::new(
            start_position.x + gap_x * i.to_f32().unwrap(),
            map_range(
                i.to_f32().unwrap().sin(),
                -1.,
                1.,
                -height_range / 2.,
                height_range / 2.,
            ),
        )))
    }
    particles
}