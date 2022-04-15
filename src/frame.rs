use crate::projectile::Projectile;

use crate::projectile::Projectile;

const FRAME_HEIGHT_PRCNT: f32 = 0.70;
const FRAME_WIDTH_PRCNT: f32 = 0.20;

const FRAME_BORDER_WIDTH: f32 = 1.0;
const FRAME_BORDER_COLOR: egui::Color32 = egui::Color32::WHITE;

const FRAME_EDGE_ROUND: f32 = 0.0;

pub struct Frame {
    window_rect: egui::Rect,
    projectile: Projectile,
    crosshair: Vec<egui::Shape>,
}

impl Frame {
    pub fn new(egui_ctx: &egui::Context, projectile: Projectile) -> Self {
        let window_rect = egui_ctx.available_rect();

        let rec_top_left =
            egui::Pos2::new(window_rect.width() * FRAME_WIDTH_PRCNT, window_rect.top());
        let rec_bottom_right = egui::Pos2::new(
            window_rect.width() * FRAME_WIDTH_PRCNT * 4.0,
            window_rect.height() * FRAME_HEIGHT_PRCNT,
        );

        let frame = egui::Rect::from_min_max(rec_top_left, rec_bottom_right);

        //let stroke = egui::epaint::Stroke::new(1.0, egui::Color32::WHITE);

        //let rect = egui::epaint::RectShape::stroke(frame, 0.0, stroke);

        Frame { window_rect: frame }
    }

    pub fn draw(&self, painter: &egui::Painter) {
        let stroke = egui::epaint::Stroke::new(FRAME_BORDER_WIDTH, FRAME_BORDER_COLOR);

        let rect = egui::epaint::RectShape::stroke(self.window_rect, FRAME_EDGE_ROUND, stroke);

        painter.add(egui::Shape::Rect(rect));
    }
}
