use std::f32;

use eframe::{egui, emath::Vec2};
use egui::Pos2;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

/// Area percentage rather than pixels
const BALL_RADIUS: f32 = 0.03;

/// Starting position which is the center of the screen.
const BALL_START_POS: Pos2 = Pos2::new(0.0, 0.0);

/// Parameter for the slow velocity, it cannot be changed within the program.
const BALL_SLOW_VELOCITY: f32 = 0.30;

/// Parameter for the medium velocity, it cannot be changed within the program.
const BALL_MEDIUM_VELOCITY: f32 = 0.60;

/// Parameter for the fast velocity, it cannot be changed within the program.
const BALL_FAST_VELOCITY: f32 = 1.20;

const BALL_NUDGE_RATE: f32 = 1.2;

/// Three types of velocity for the ball
/// By default the one that will be used the most is Slow as is the one that makes it easy to
/// handle.
/// The reason for this type is to have more freedom on the velocity for future use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BallVelocity {
    /// Slowest velocity and default by use [`BALL_SLOW_VELOCITY`]
    #[serde(rename = "slow")]
    Slow,
    /// Medium velocity [`BALL_MEDIUM_VELOCITY`]
    #[serde(rename = "medium")]
    Medium,
    /// Fast velocity [`BALL_FAST_VELOCITY`]
    #[serde(rename = "fast")]
    Fast,
}

pub struct Ball {
    /// Current position in the screen.
    ///
    /// We use a default range of -1.0 to 1.0 where 0.0 is the center of the screen.
    /// This is later then factored to the monitor's dimension to be scale it properly.
    pos: Pos2,

    velocity: Vec2,

    /// Keeps track of how long has the ball been running since last velocity change
    time_running: f32,

    /// Given by the config file, there will be a random time where the ball is suppose to change
    /// it's velocity. This could be any times of seconds given by a range.
    ///
    /// For example any time from 1 second to 7 seconds.
    velocity_change_time_at: f32,

    /// The minimum value of the time range
    pub random_direction_change_time_min: f32,

    /// The maximum value of the time range
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
    pub fn update(&mut self, input_axes: Vec2, delta_time: f32) {
        let mut rng = rand::thread_rng();

        if self.time_running >= self.velocity_change_time_at {
            self.velocity = new_vel(self.velocity.length());

            self.time_running = 0.0;
            self.velocity_change_time_at = rng.gen_range(
                self.random_direction_change_time_min..=self.random_direction_change_time_max,
            );
        }

        self.pos.x += self.velocity.x * delta_time;
        self.pos.y += self.velocity.y * delta_time;

        // Based on input
        self.pos.x += input_axes.x * BALL_NUDGE_RATE * delta_time;
        // Corrects for the fact that positive y here is down
        self.pos.y -= input_axes.y * BALL_NUDGE_RATE * delta_time;

        // This is for bounds checking on the ball
        // The addition or subtraction inside the logic is so the circle does not use the center as
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
/// Function to make calculate the new velocity.
/// This uses geometry where it can choose any velocity within a 360 degree angle of the ball.
fn new_vel(length: f32) -> Vec2 {
    let mut rng = rand::thread_rng();

    // Some how by MAF, changing the angle between -pi/2 to pi/2 makes it so the ball nevers goes towards the center of the screen.
    // To be fair I'm not math major so i don't have a solid explanation for this. I'm thinking is just because the ball will go perpendicular to its position.
    let radians = rng.gen_range(-f32::consts::FRAC_PI_2..f32::consts::FRAC_PI_2);

    let (x, y) = (radians.cos(), radians.sin());

    Vec2::new(x * length, y * length)
}

impl Default for Ball {
    fn default() -> Self {
        Self::new(0.0, 0.0, BallVelocity::Slow)
    }
}
