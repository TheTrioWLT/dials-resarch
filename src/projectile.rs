use egui::epaint::CircleShape;
use egui::Pos2;

const RADIUS: f32 = 5.0;
const X_VEL: f32 = 3.0;
const Y_VEL: f32 = 5.0;

const BALL_COLOR: egui::Color32 = egui::Color32::LIGHT_GREEN;

pub struct ProjectileDrawData {
    pub frame: egui::Rect,
    pub width_pos: Pos2,
    pub height_pos: Pos2,
}

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    dx: f32,
    dy: f32,
    radius: f32,
    fill_color: egui::Color32,
}

impl Projectile {
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Projectile {
            x,
            y,
            radius,
            fill_color: BALL_COLOR,
            dx: X_VEL,
            dy: Y_VEL,
        }
    }

    pub fn draw(&mut self, painter: &egui::Painter, draw_data: &ProjectileDrawData) {
        self.x = self.x - self.dx;
        self.y = self.y - self.dy;

        if self.x >= draw_data.frame.max.x || self.x <= draw_data.frame.min.x {
            //Check bound collition given a frame
            self.dx *= -1.0;
        }

        if self.y <= draw_data.frame.min.y || self.y >= draw_data.frame.max.y {
            self.dy *= -1.0;
        }

        painter.add(egui::Shape::Circle(CircleShape::filled(
            Pos2::new(self.x, self.y),
            self.radius,
            BALL_COLOR,
        )));
    }

    pub fn centered(&mut self, painter: &egui::Painter, draw_data: &ProjectileDrawData) {
        self.x = draw_data.frame.center().x;
        self.y = draw_data.frame.center().y;

        painter.add(egui::Shape::Circle(CircleShape::filled(
            Pos2::new(self.x, self.y),
            self.radius,
            self.fill_color,
        )));
    }
}

impl Default for Projectile {
    fn default() -> Self {
        Projectile {
            x: 0.0,
            y: 0.0,
            dx: X_VEL,
            dy: Y_VEL,
            radius: RADIUS,
            fill_color: BALL_COLOR,
        }
    }
}
