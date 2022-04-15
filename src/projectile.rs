use egui::epaint::CircleShape;
use egui::Pos2;
use rand::Rng;
use std::time;

const PROJ_SPEED: f32 = 5.0;

pub struct ProjectileDrawData {
    pub radius: f32,
    pub width_pos: Pos2,
    pub height_pos: Pos2,
    pub origin: Pos2,
    pub max_range: Pos2,
}

pub struct Projectile {
    pub x: f32,
    pub y: f32,
    radius: f32,
}

impl Projectile {
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Projectile { x, y, radius }
    }

    pub fn draw(&mut self, painter: &egui::Painter, draw_data: &ProjectileDrawData) {
        let mut range = rand::thread_rng();

        let dest_x = range.gen_range(draw_data.origin.x..draw_data.max_range.x);
        let dest_y = range.gen_range(draw_data.origin.y..draw_data.max_range.y);

        let center = Pos2::new(dest_x, dest_y);
        if (center.x >= draw_data.origin.x && center.y >= draw_data.origin.y)
            && (center.x <= draw_data.max_range.x && center.y <= draw_data.max_range.y)
        {
            painter.add(egui::Shape::Circle(CircleShape::filled(
                center,
                draw_data.radius,
                egui::Color32::LIGHT_GREEN,
            )));
        }
    }
}

impl Default for Projectile {
    fn default() -> Self {
        Projectile {
            x: 0.0,
            y: 0.0,
            radius: 5.0,
        }
    }
}
