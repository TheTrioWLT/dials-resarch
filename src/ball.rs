use std::f32;

use eframe::{egui, emath::Vec2};
use egui::Pos2;
use rand::prelude::*;

// Area percentage rather than pixels
const BALL_RADIUS: f32 = 0.03;

const BALL_START_POS: Pos2 = Pos2::new(0.0, 0.0);

//
const BALL_SLOW_VELOCITY: f32 = 0.75;
const BALL_MEDIUM_VELOCITY: f32 = 1.0;
const BALL_FAST_VELOCITY: f32 = 1.25;

const BALL_NUDGE_RATE: f32 = 1.2;

#[derive(Debug, Clone, Copy)]
pub enum BallVelocity {
    Slow,
    Medium,
    Fast,
}
pub struct Ball {
    pos: Pos2,
    velocity: Vec2,
    time_running: f32,
    velocity_change_time_at: f32,
    pub random_direction_change_time_min: f32,
    pub random_direction_change_time_max: f32,
}

impl Ball {
    /// Creates a new ball that begins in the default starting position with the ball's correct
    /// starting velocity
    pub fn new(
        random_direction_change_time_min: f32,
        random_direction_change_time_max: f32,
        velocity_meter: BallVelocity,
    ) -> Self {
        let length = match velocity_meter {
            BallVelocity::Slow => BALL_SLOW_VELOCITY,
            BallVelocity::Medium => BALL_MEDIUM_VELOCITY,
            BallVelocity::Fast => BALL_FAST_VELOCITY,
        };

        Self {
            pos: BALL_START_POS,
            velocity: new_vel(length),
            time_running: 0.0,
            velocity_change_time_at: 0.0,
            random_direction_change_time_min,
            random_direction_change_time_max,
        }
    }

    /// Movement of ball through the 2D plane
    ///
    /// The coordinate system used is (-1.0, -1.0) to (1.0, 1.0)
    ///
    /// Where -1.0 is the minimum of x or y and 1.0 is the maximum of x and y.
    ///
    /// This is was done with the purpose of being able to draw the initial position of the
    /// projectile (center) without the use of the screen dimensions.
    ///
    /// The center of the screen would be the (screen_width / 2, screen_height / 2) this can be
    /// translated to (0.0, 0.0).
    ///
    pub fn update(&mut self, input_axes: Vec2, delta_time: f32) {
        let mut rng = rand::thread_rng();

        if self.time_running >= self.velocity_change_time_at {
            self.velocity = new_vel(self.velocity.length());

            self.time_running = 0.0;
            self.velocity_change_time_at = rng.gen_range(
                self.random_direction_change_time_min..=self.random_direction_change_time_max,
            ) as f32;
        }

        self.pos.x += self.velocity.x * delta_time;
        self.pos.y += self.velocity.y * delta_time;

        // Based on input
        self.pos.x += input_axes.x * BALL_NUDGE_RATE * delta_time;
        // Corrects for the fact that positive y here is down
        self.pos.y -= input_axes.y * BALL_NUDGE_RATE * delta_time;

        // This is for bounds checking on the ball
        // The addition or substraction inside the logic is so the circle does not use the center as
        // the x or y location. This way the circle would not go through some of the borders.

        if (self.pos.x + BALL_RADIUS) >= 1.0 {
            self.pos.x = 1.0 - BALL_RADIUS;
            self.velocity.x = -self.velocity.x.abs();
        }
        if (self.pos.x - BALL_RADIUS) <= -1.0 {
            self.pos.x = -1.0 + BALL_RADIUS;
            self.velocity.x = self.velocity.x.abs();
        }

        if (self.pos.y - BALL_RADIUS) <= -1.0 {
            self.pos.y = -1.0 + BALL_RADIUS;
            self.velocity.y = self.velocity.y.abs();
        }
        if (self.pos.y + BALL_RADIUS) >= 1.0 {
            self.pos.y = 1.0 - BALL_RADIUS;
            self.velocity.y = -self.velocity.y.abs();
        }

        self.time_running += delta_time;
    }

    pub fn current_rms_error(&self) -> f32 {
        self.pos.x.powf(2.0) + self.pos.y.powf(2.0) // Distance from the center squared
    }

    pub fn pos(&self) -> Pos2 {
        self.pos
    }
}
//Function to make calculate the new velocity
fn new_vel(length: f32) -> Vec2 {
    let mut rng = rand::thread_rng();

    let radians = rng.gen_range(0.0..2.0 * f32::consts::PI);

    let (x, y) = (radians.cos(), radians.sin());

    Vec2::new(x * length, y * length)
}

impl Default for Ball {
    fn default() -> Self {
        Self::new(0.0, 0.0, BallVelocity::Slow)
    }
}
