use nannou::prelude::{map_range, ToPrimitive};

pub trait Animation {
    fn new(duration: u16, start_brightness: f32, start_time: u128) -> Self;
    fn duration(&self) -> u16;
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

pub struct Attack {
    start_time: u128,
    duration: u16,
    range: (f32, f32),
}

impl Animation for Attack {
    fn new(duration: u16, start_brightness: f32, start_time: u128) -> Self {
        Attack {
            start_time,
            duration,
            range: (start_brightness, 1.0),
        }
    }

    fn start_time(&self) -> u128 {
        self.start_time
    }
    fn duration(&self) -> u16 {
        self.duration
    }
    fn get_range(&self) -> (f32, f32) {
        self.range
    }
}
pub struct Release {
    start_time: u128,
    duration: u16,
    range: (f32, f32),
}

impl Animation for Release {
    fn new(duration: u16, start_brightness: f32, start_time: u128) -> Self {
        Release {
            start_time,
            duration,
            range: (start_brightness, 0.0),
        }
    }
    fn start_time(&self) -> u128 {
        self.start_time
    }
    fn duration(&self) -> u16 {
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
