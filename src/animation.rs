use nannou::prelude::ToPrimitive;
use tween::{Tween, Tweener};

type StoredTweener = Tweener<f32, usize, Box<dyn Tween<f32>>>;

pub trait Animation {
    fn new(
        duration: usize,
        start_brightness: f32,
        target_brightness: f32,
        tween: Box<dyn Tween<f32>>,
    ) -> Self;
    fn duration(&self) -> usize;

    fn get_elapsed(&self) -> i64;
    fn set_elapsed(&mut self, time: i64);

    fn get_tweener(&mut self) -> &mut StoredTweener;

    /// Update the animation using delta time, get the progress in the range `[0,1]`
    /// Only return a new progress value if the animation is actually updating;
    /// for example when there is a delayed start, leave the progress alone
    fn update(&mut self, delta_time: usize) -> Option<f32> {
        let elapsed = self.get_elapsed() + delta_time.to_i64().unwrap();
        self.set_elapsed(elapsed);

        if elapsed >= 0 {
            // let progress = elapsed.to_f64().unwrap() / self.duration().to_f64().unwrap();
            let progress = self.get_tweener().move_by(delta_time);
            Some(progress)
        } else {
            None
        }
    }

    fn get_brightness_and_done(&mut self, delta_time: usize) -> (f32, bool) {
        if let Some(brightness) = self.update(delta_time) {
            (brightness, self.get_tweener().is_finished())
        } else {
            (self.get_tweener().initial_value(), false)
        }
    }
}

pub struct Attack {
    elapsed: i64,
    duration: usize,
    tweener: StoredTweener,
}

impl Animation for Attack {
    fn new(
        duration: usize,
        start_brightness: f32,
        target_brightness: f32,
        tween: Box<dyn Tween<f32>>,
    ) -> Self {
        Attack {
            elapsed: 0,
            duration,
            tweener: Tweener::new(start_brightness, target_brightness, duration, tween),
        }
    }

    fn get_elapsed(&self) -> i64 {
        self.elapsed
    }

    fn set_elapsed(&mut self, time: i64) {
        self.elapsed = time;
    }

    fn get_tweener(&mut self) -> &mut StoredTweener {
        &mut self.tweener
    }

    fn duration(&self) -> usize {
        self.duration
    }
}
pub struct Release {
    elapsed: i64,
    duration: usize,
    tweener: StoredTweener,
}

impl Animation for Release {
    fn new(
        duration: usize,
        start_brightness: f32,
        target_brightness: f32,
        tween: Box<dyn Tween<f32>>,
    ) -> Self {
        Release {
            elapsed: 0,
            duration,
            tweener: Tweener::new(start_brightness, target_brightness, duration, tween),
        }
    }
    fn get_elapsed(&self) -> i64 {
        self.elapsed
    }
    fn set_elapsed(&mut self, time: i64) {
        self.elapsed = time;
    }
    fn get_tweener(&mut self) -> &mut StoredTweener {
        &mut self.tweener
    }
    fn duration(&self) -> usize {
        self.duration
    }
}

// The animation concept is based on https://en.wikipedia.org/wiki/Envelope_(music)
pub enum EnvelopeStage {
    // Active(Box<dyn Animated>),
    AttackAnimation(Attack),
    ReleaseAnimation(Release),
    Idle(),
}
