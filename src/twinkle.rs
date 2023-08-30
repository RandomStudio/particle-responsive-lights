use nannou::prelude::{ToPrimitive, PI};

pub struct Twinkle {
    elapsed: i64,
    duration: usize,
    offset: usize,
    min_brightness: f32,
    max_brightness: f32,
    cur_brightness: f32,
}

impl Twinkle {
    pub fn new(duration: usize, offset: usize, min_brightness: f32, max_brightness: f32) -> Self {
        Self {
            elapsed: 0,
            duration,
            offset,
            min_brightness,
            max_brightness,
            cur_brightness: 0.0,
        }
    }

    pub fn update(&mut self, time: usize) {
        let d = self.duration.to_f32().unwrap();
        let elapsed = (time + self.offset * self.duration).to_f32().unwrap();
        let cos = 0.5 + 0.5 * (2.0 * PI * (elapsed % d) / d).cos();
        self.cur_brightness = cos * cos * cos * cos;
    }

    pub fn get_brightness(&self) -> f32 {
        self.cur_brightness
    }
}
