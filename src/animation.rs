use nannou::prelude::ToPrimitive;
use tween::{Tween, Tweener};

type StoredTweener = Tweener<f32, usize, Box<dyn Tween<f32>>>;

pub struct Animation {
    pub elapsed: i64,
    pub duration: usize,
    pub tweener: StoredTweener,
}

impl Animation {
    pub fn new(
        duration: usize,
        start_brightness: f32,
        target_brightness: f32,
        tween: Box<dyn Tween<f32>>,
    ) -> Self {
        Animation {
            duration,
            elapsed: 0,
            tweener: Tweener::new(start_brightness, target_brightness, duration, tween),
        }
    }

    /// It is allowed to set negative elapsed values on an Animation,
    /// for example to "wait" before starting the tween (delay start)
    pub fn set_elapsed(&mut self, elapsed: i64) {
        self.elapsed = elapsed;
    }

    /// Update the animation using delta time, get the progress in the range `[0,1]`
    /// Only return a new progress value if the animation is actually updating;
    /// for example when there is a delayed start, leave the progress alone
    fn update(&mut self, delta_time: usize) -> Option<f32> {
        self.elapsed += delta_time.to_i64().unwrap();
        let elapsed = self.elapsed;

        if elapsed >= 0 {
            // let progress = elapsed.to_f64().unwrap() / self.duration().to_f64().unwrap();
            let progress = self.tweener.move_by(delta_time);
            Some(progress)
        } else {
            None
        }
    }

    pub fn get_brightness_and_done(&mut self, delta_time: usize) -> (f32, bool) {
        if let Some(brightness) = self.update(delta_time) {
            (brightness, self.tweener.is_finished())
        } else {
            (self.tweener.initial_value(), false)
        }
    }
}

pub struct AfterAttack {
    pub release_duration: usize,
    pub final_brightness: f32,
}

// The animation concept is based on https://en.wikipedia.org/wiki/Envelope_(music)
pub enum EnvelopeStage {
    /// The Attack animation to play, followed by the
    /// (optional) duration and final brightness of the
    /// Release animation that follows
    AttackAnimation(Animation, Option<AfterAttack>),
    ReleaseAnimation(Animation),
    Idle(),
}
