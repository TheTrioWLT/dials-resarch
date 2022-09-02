use eframe::{egui, emath::Vec2};
use egui::Pos2;

// Area percentage rather than pixels
const BALL_RADIUS: f32 = 0.03;

const BALL_START_POS: Pos2 = Pos2::new(0.0, 0.0);

// TODO: Move to be read from the configuration file!
// This is specified in the normalized vector position units per second
const BALL_START_VELOCITY: Vec2 = Vec2::new(0.25, 0.35);

const BALL_NUDGE_RATE: f32 = 0.003;

pub struct Ball {
    pos: Pos2,
    velocity: Vec2,
}

impl Ball {
    /// Creates a new ball that begins in the default starting position with the ball's correct
    /// starting velocity
    pub fn new() -> Self {
        Self {
            pos: BALL_START_POS,
            velocity: BALL_START_VELOCITY,
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
        // Update the ball's position
        self.pos.x += self.velocity.x * delta_time;
        self.pos.y += self.velocity.y * delta_time;

        self.pos.x += input_axes.x * BALL_NUDGE_RATE;
        // Corrects for the fact that positive y here is down
        self.pos.y -= input_axes.y * BALL_NUDGE_RATE;

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
    }

    pub fn pos(&self) -> Pos2 {
        self.pos
    }
}

impl Default for Ball {
    fn default() -> Self {
        Self::new()
    }
}
