use egui::epaint::CircleShape;
use egui::Pos2;

const BALL_RADIUS: f32 = 5.0;

const BALL_COLOR: egui::Color32 = egui::Color32::LIGHT_GREEN;

const MIN_VALUE: f32 = 0.0;
const MAX_VALUE: f32 = 1.0;

const BALL_START_POS: Pos2 = Pos2::new(0.0, 0.0);

// TODO: Move to be read from the configuration file!
// This is specified in the normalized vector position units per second
const BALL_START_VELOCITY: Pos2 = Pos2::new(0.005, 0.007);

pub struct Ball {
    pos: Pos2,
    velocity: Pos2,
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
    pub fn draw(&mut self, painter: &egui::Painter, frame: &egui::Rect, delta_time: f32) {
        let screen_x = frame.min.x + (self.pos.x * frame.width());
        let screen_y = frame.min.y + (self.pos.y * frame.height());

        //The addition or substraction inside the logic is so the circle does not use the center as
        //the x or y location. This way the circle would not go thru some of the borders.
        if (self.velocity.x + 0.01) >= MAX_VALUE {
            //Check bound collition given a frame
            self.velocity.x *= -1.0;
        }
        if (self.velocity.x - 0.01) <= MIN_VALUE {
            self.velocity.x *= -1.0;
        }

        if (self.velocity.y - 0.01) <= MIN_VALUE {
            self.velocity.y *= -1.0;
        }

        if (self.velocity.y + 0.01) >= MAX_VALUE {
            self.velocity.y *= -1.0;
        }

        painter.add(egui::Shape::Circle(CircleShape::filled(
            Pos2::new(screen_x, screen_y),
            BALL_RADIUS,
            BALL_COLOR,
        )));

        // Update the ball's position
        self.pos.x += self.velocity.x * delta_time;
        self.pos.y += self.velocity.y * delta_time;
    }
}

impl Default for Ball {
    fn default() -> Self {
        Self::new()
    }
}
