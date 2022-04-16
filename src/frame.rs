use crate::ball::Ball;
use egui::{Color32, Pos2};

const FRAME_BORDER_WIDTH: f32 = 1.0;
const FRAME_BORDER_COLOR: Color32 = Color32::WHITE;

const FRAME_MARGIN_PERCENT: f32 = 0.0125;
const FRAME_MAX_WIDTH_PERCENT: f32 = 1.0 - 2.0 * FRAME_MARGIN_PERCENT;
const FRAME_MAX_HEIGHT_PERCENT: f32 = 0.70 - 2.0 * FRAME_MARGIN_PERCENT;

// These positions are in virtual coordinates where the coordinates are normalized based on the
// size of the ball's frame
const CROSSHAIR_START_POS: Pos2 = Pos2::new(0.0, 0.0);
// This is a percentage of the *frame size*, not window size
const CROSSHAIR_SIZE_PERCENT: f32 = 0.125;
const CROSSHAIR_STROKE: f32 = 1.0;
const CROSSHAIR_COLOR: Color32 = Color32::WHITE;

pub struct Crosshair {
    pos: egui::Pos2,
}

impl Crosshair {
    pub fn new() -> Self {
        Self {
            pos: CROSSHAIR_START_POS,
        }
    }

    pub fn set_x(&mut self, x: f32) {
        self.pos.x = x;
    }

    pub fn set_y(&mut self, y: f32) {
        self.pos.y = y;
    }

    pub fn increment_x(&mut self, x: f32) {
        self.pos.x += x;
    }

    pub fn increment_y(&mut self, y: f32) {
        self.pos.y += y;
    }

    pub fn position(&mut self) -> Pos2 {
        self.pos
    }

    pub fn draw(&self, painter: &egui::Painter, frame_rect: &egui::Rect) {
        let frame_center = frame_rect.center();
        // The frame is guaranteed to be square
        let crosshair_half_size = CROSSHAIR_SIZE_PERCENT * frame_rect.height() / 2.0;

        let v_top_pos = Pos2::new(frame_center.x, frame_center.y - crosshair_half_size);
        let v_bottom_pos = Pos2::new(frame_center.x, frame_center.y + crosshair_half_size);

        let h_left_pos = Pos2::new(frame_center.x - crosshair_half_size, frame_center.y);
        let h_right_pos = Pos2::new(frame_center.x + crosshair_half_size, frame_center.y);

        let stroke = egui::Stroke::new(CROSSHAIR_STROKE, CROSSHAIR_COLOR);

        painter.line_segment([v_top_pos, v_bottom_pos], stroke);
        painter.line_segment([h_left_pos, h_right_pos], stroke);

        /*
        // Making the lines for the crosshair
        //
        // Needs fixing, this way of constructing is not ideal
        let v_top_pos = egui::Pos2::new(
            frame.center().x,
            (frame.center().y - frame.height() * V_CROSSHAIR_OFFSET).abs(),
        );
        let v_bot_pos = egui::Pos2::new(
            frame.center().x,
            frame.center().y + frame.height() * V_CROSSHAIR_OFFSET,
        );

        crosshair.push(egui::Shape::LineSegment {
            points: [v_top_pos, v_bot_pos],
            stroke,
        });

        let h_left_pos = egui::Pos2::new(
            (frame.center().x - frame.width() * H_CROSSHAIR_OFFSET).abs(),
            frame.center().y,
        );

        let h_right_pos = egui::Pos2::new(
            frame.center().x + frame.width() * H_CROSSHAIR_OFFSET,
            frame.center().y,
        );

        crosshair.push(egui::Shape::LineSegment {
            points: [h_left_pos, h_right_pos],
            stroke,
        });
        */
    }
}

impl Default for Crosshair {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds the rectangle in which the ball moves in the main tracking task
pub struct Frame {
    pub crosshair: Crosshair,
    pub ball: Ball,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            crosshair: Crosshair::new(),
            ball: Ball::new(),
        }
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

        self.crosshair.draw(painter, &frame_rect);
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}
