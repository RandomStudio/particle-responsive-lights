use nannou::prelude::{map_range, ToPrimitive};

pub trait Animation {
    fn new(duration: usize, start_brightness: f32) -> Self;
    fn duration(&self) -> usize;

    fn get_time_since_start(&self) -> usize;
    fn set_time_since_start(&mut self, time: usize);

    fn get_range(&self) -> (f32, f32);

    /// Update the animation using delta time, get the progress in the range `[0,1]`
    fn update(&mut self, delta_time: usize) -> f32 {
        let elapsed = self.get_time_since_start() + delta_time;
        self.set_time_since_start(elapsed);
        let progress = elapsed.to_f64().unwrap() / self.duration().to_f64().unwrap();
        progress.to_f32().unwrap()
    }

    fn get_brightness_and_done(&mut self, delta_time: usize) -> (f32, bool) {
        let (from, to) = self.get_range();
        let progress = self.update(delta_time);
        let brightness = map_range(progress, 0., 1., from, to);
        if progress > 1. {
            (to, true)
        } else {
            (brightness, false)
        }
    }
}

pub struct Attack {
    elapsed: usize,
    duration: usize,
    range: (f32, f32),
}

impl Animation for Attack {
    fn new(duration: usize, start_brightness: f32) -> Self {
        Attack {
            elapsed: 0,
            duration,
            range: (start_brightness, 1.0),
        }
    }

    fn get_time_since_start(&self) -> usize {
        self.elapsed
    }

    fn set_time_since_start(&mut self, time: usize) {
        self.elapsed = time;
    }

    fn duration(&self) -> usize {
        self.duration
    }
    fn get_range(&self) -> (f32, f32) {
        self.range
    }
}
pub struct Release {
    elapsed: usize,
    duration: usize,
    range: (f32, f32),
}

impl Animation for Release {
    fn new(duration: usize, start_brightness: f32) -> Self {
        Release {
            elapsed: 0,
            duration,
            range: (start_brightness, 0.0),
        }
    }
    fn get_time_since_start(&self) -> usize {
        self.elapsed
    }
    fn set_time_since_start(&mut self, time: usize) {
        self.elapsed = time;
    }
    fn duration(&self) -> usize {
        self.duration
    }
    fn get_range(&self) -> (f32, f32) {
        self.range
    }
}

// The animation concept is based on https://en.wikipedia.org/wiki/Envelope_(music)
pub enum EnvelopeStage {
    // Active(Box<dyn Animated>),
    AttackAnimation(Attack),
    ReleaseAnimation(Release),
    Idle(),
}
