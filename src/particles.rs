use log::debug;
use nannou::{
    prelude::{map_range, Point2, ToPrimitive},
    rand::{random, random_range},
};

use crate::{animation::EnvelopeStage, settings::DEFAULT_COUNT, twinkle::Twinkle};

pub struct Particle {
    pub id: usize,
    pub order: usize,
    pub position: Point2,
    brightness: f32,
    pub animation: EnvelopeStage,
    twinkler: Twinkle,
}

impl Particle {
    fn new(id: usize, order: usize, position: Point2) -> Self {
        Particle {
            id,
            order,
            position,
            brightness: 0.,
            animation: EnvelopeStage::Idle(),
            twinkler: Twinkle::new(
                random_range(5500, 12500),
                random_range(0, 9000),
                0.002,
                0.04,
            ),
        }
    }
    pub fn brightness(&self) -> f32 {
        let tb = self.twinkler.get_brightness();
        if tb > self.brightness {
            tb.clamp(0.0, 1.0)
        } else {
            self.brightness.clamp(0., 1.0)
        }
    }
    pub fn set_brightness(&mut self, new_value: f32) {
        self.brightness = new_value;
    }

    pub fn twinkle(&mut self, time: usize) {
        self.twinkler.update(time);
    }
}

pub fn build_layout(
    count: usize,
    width_range: f32,
    height_range: f32,
    order: &[usize; DEFAULT_COUNT],
) -> Vec<Particle> {
    let gap_x = width_range / count.to_f32().unwrap();
    let start_position = Point2::new(-width_range / 2. + gap_x / 2., -height_range / 2.);
    let mut particles: Vec<Particle> = vec![];
    for (i, id) in order.iter().enumerate().take(count) {
        debug!("assign order {id} to ID #{i}");
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
