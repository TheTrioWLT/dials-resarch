use egui::epaint::CircleShape;
use egui::Pos2;

const RADIUS: f32 = 5.0;
const X_VEL: f32 = 0.005;
const Y_VEL: f32 = 0.007;

const BALL_COLOR: egui::Color32 = egui::Color32::LIGHT_GREEN;

const MIN_VALUE: f32 = 0.0;
const MAX_VALUE: f32 = 1.0;

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    dx: f32,
    dy: f32,
    radius: f32,
    fill_color: egui::Color32,
}

impl Projectile {
    //pub fn new(x: f32, y: f32, radius: f32) -> Self {
    //    Projectile {
    //        x,
    //        y,
    //        radius,
    //        fill_color: BALL_COLOR,
    //        dx: X_VEL,
    //        dy: Y_VEL,
    //    }
    //}

    ///Movement of ball thru the 2D plane
    ///
    ///The coordinate system used is (0.0,0.0)--(1.0,1.0)
    ///Where 0.0 is the minimum x or y and 1.0 is the maximum of any of this coordinates.
    ///
    ///This is was done with the purpose of being able to draw the initial position of the
    ///projectile(center) with out the use of the screen parameters at first.
    ///
    ///The center of the screen would be the (screen_width/2, screen_height/2) this can be
    ///translated to (0.5, 0.5).
    pub fn draw(&mut self, painter: &egui::Painter, frame: &egui::Rect) {
        self.x = self.x + self.dx;
        self.y = self.y - self.dy;

        let screen_x = frame.min.x + (self.x * frame.width());
        let screen_y = frame.min.y + (self.y * frame.height());

        //The addition or substraction inside the logic is so the circle does not use the center as
        //the x or y location. This way the circle would not go thru some of the borders.
        if (self.x + 0.01) >= MAX_VALUE {
            //Check bound collition given a frame
            self.dx *= -1.0;
        }
        if (self.x - 0.01) <= MIN_VALUE {
            self.dx *= -1.0;
        }

        if (self.y - 0.01) <= MIN_VALUE {
            self.dy *= -1.0;
        }

        if (self.y + 0.01) >= MAX_VALUE {
            self.dy *= -1.0;
        }

        painter.add(egui::Shape::Circle(CircleShape::filled(
            Pos2::new(screen_x, screen_y),
            self.radius,
            self.fill_color,
        )));
    }

    ////Centers the projectile given window size
    //pub fn centered(&mut self, frame: &egui::Rect) {
    //    self.x = frame.center().x;
    //    self.y = frame.center().y;
    //}
}

///Default values to put circle in center
impl Default for Projectile {
    fn default() -> Self {
        Projectile {
            x: 0.5,
            y: 0.5,
            dx: X_VEL,
            dy: Y_VEL,
            radius: RADIUS,
            fill_color: BALL_COLOR,
        }
    }
}
