use eframe::{
    egui,
    emath::{Pos2, Vec2},
    epaint::{CircleShape, Color32},
};
use derive_new::new;

const FRAME_BORDER_WIDTH: f32 = 1.0;
const FRAME_BORDER_COLOR: Color32 = Color32::WHITE;

const FRAME_MAX_HEIGHT_PERCENT: f32 = 1.0;
const FRAME_MAX_WIDTH_PERCENT: f32 = 0.7;

// This is a percentage of the *frame size*, not window size
const CROSSHAIR_SIZE_PERCENT: f32 = 0.125;
const CROSSHAIR_STROKE: f32 = 1.0;
const CROSSHAIR_COLOR: Color32 = Color32::WHITE;

const BALL_RADIUS: f32 = 0.03;

const BALL_COLOR: egui::Color32 = egui::Color32::LIGHT_GREEN;

#[derive(new)]
pub struct TrackingWidget {
    ball_pos: Pos2,
}

impl TrackingWidget {
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let height_size = FRAME_MAX_HEIGHT_PERCENT * ui.available_height();
        let width_size = FRAME_MAX_WIDTH_PERCENT * ui.available_width();

        let min_both = height_size.min(width_size);

        let desired_size = Vec2::splat(min_both);
        let (rect, mut response) =
            ui.allocate_exact_size(desired_size, egui::Sense::focusable_noninteractive());

        // Only draw if we need to
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Draw the frame
            let frame_shape = egui::epaint::RectShape::stroke(
                rect,
                egui::Rounding::none(),
                egui::Stroke::new(FRAME_BORDER_WIDTH, FRAME_BORDER_COLOR),
            );

            painter.add(egui::Shape::Rect(frame_shape));

            // Draw the crosshair
            // The frame is guaranteed to be square
            let frame_width = rect.width();
            let crosshair_half_size = CROSSHAIR_SIZE_PERCENT * frame_width / 2.0;
            let center = rect.center();

            let v_top_pos = Pos2::new(center.x, center.y - crosshair_half_size);
            let v_bottom_pos = Pos2::new(center.x, center.y + crosshair_half_size);

            let h_left_pos = Pos2::new(center.x - crosshair_half_size, center.y);
            let h_right_pos = Pos2::new(center.x + crosshair_half_size, center.y);

            let stroke = egui::Stroke::new(CROSSHAIR_STROKE, CROSSHAIR_COLOR);

            painter.line_segment([v_top_pos, v_bottom_pos], stroke);
            painter.line_segment([h_left_pos, h_right_pos], stroke);

            // Draw the ball
            let half_frame_width = frame_width / 2.0;

            let ball_center = Pos2::new(
                center.x + self.ball_pos.x * half_frame_width,
                center.y + self.ball_pos.y * half_frame_width,
            );

            let ball_pixel_radius = BALL_RADIUS * half_frame_width;

            painter.add(egui::Shape::Circle(CircleShape::filled(
                ball_center,
                ball_pixel_radius,
                BALL_COLOR,
            )));
        }

        response.mark_changed();

        response
    }
}
