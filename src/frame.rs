use crate::ball::Ball;
use egui::{Color32, Pos2};

const FRAME_BORDER_WIDTH: f32 = 1.0;
const FRAME_BORDER_COLOR: Color32 = Color32::WHITE;

const FRAME_MARGIN_PERCENT: f32 = 0.0125;
const FRAME_MAX_WIDTH_PERCENT: f32 = 1.0 - 2.0 * FRAME_MARGIN_PERCENT;
const FRAME_MAX_HEIGHT_PERCENT: f32 = 0.70 - 2.0 * FRAME_MARGIN_PERCENT;

// This is a percentage of the *frame size*, not window size
const CROSSHAIR_SIZE_PERCENT: f32 = 0.125;
const CROSSHAIR_STROKE: f32 = 1.0;
const CROSSHAIR_COLOR: Color32 = Color32::WHITE;

/// Holds the rectangle in which the ball moves in the main tracking task
pub struct Frame {
    pub ball: Ball,
}

impl Frame {
    pub fn new() -> Self {
        Self { ball: Ball::new() }
    }

    pub fn update(&mut self, input_axes: &egui::Pos2, delta_time: f32) {
        self.ball.update(input_axes, delta_time);
    }

    /// Draws the frame
    pub fn draw(&mut self, painter: &egui::Painter, window_rect: &egui::Rect) {
        let window_center_top = window_rect.center_top();

        let window_width = window_rect.width();
        let window_height = window_rect.height();

        // The frame's side length as determined by the width of the window
        let frame_width_by_width = window_width * FRAME_MAX_WIDTH_PERCENT;

        // The frame's side length as determined by the height of the window
        let frame_width_by_height = window_height * FRAME_MAX_HEIGHT_PERCENT;

        // We need whichever is the smallest because we need it to fit properly in the window
        let frame_width = frame_width_by_width.min(frame_width_by_height);
        let frame_half_width = frame_width / 2.0;

        let frame_top_offset = window_height * FRAME_MARGIN_PERCENT;

        let frame_center = Pos2::new(window_center_top.x, frame_top_offset + frame_half_width);
        let frame_rect = egui::Rect::from_center_size(
            frame_center,
            egui::Vec2 {
                x: frame_width,
                y: frame_width,
            },
        );

        let frame_rect_shape = egui::epaint::RectShape::stroke(
            frame_rect,
            egui::Rounding::none(),
            egui::Stroke::new(FRAME_BORDER_WIDTH, FRAME_BORDER_COLOR),
        );

        painter.add(egui::Shape::Rect(frame_rect_shape));

        // Draw the ball
        self.ball.draw(painter, &frame_rect);

        // Draw the crosshair
        // The frame is guaranteed to be square
        let frame_width = frame_rect.width();
        let crosshair_half_size = CROSSHAIR_SIZE_PERCENT * frame_width / 2.0;

        let v_top_pos = Pos2::new(frame_center.x, frame_center.y - crosshair_half_size);
        let v_bottom_pos = Pos2::new(frame_center.x, frame_center.y + crosshair_half_size);

        let h_left_pos = Pos2::new(frame_center.x - crosshair_half_size, frame_center.y);
        let h_right_pos = Pos2::new(frame_center.x + crosshair_half_size, frame_center.y);

        let stroke = egui::Stroke::new(CROSSHAIR_STROKE, CROSSHAIR_COLOR);

        painter.line_segment([v_top_pos, v_bottom_pos], stroke);
        painter.line_segment([h_left_pos, h_right_pos], stroke);
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}
