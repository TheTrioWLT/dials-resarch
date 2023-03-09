use derive_new::new;
use eframe::{
    egui,
    emath::{Align2, Pos2, Vec2},
    epaint::{CircleShape, Color32, FontId},
};
use serde::{Deserialize, Serialize};

const FRAME_BORDER_WIDTH: f32 = 1.0;
const FRAME_BORDER_COLOR: Color32 = Color32::WHITE;

const FRAME_MAX_HEIGHT_PERCENT: f32 = 1.0;
const FRAME_MAX_WIDTH_PERCENT: f32 = 0.7;

const CROSSHAIR_STROKE: f32 = 1.0;
const CROSSHAIR_COLOR: Color32 = Color32::WHITE;

const BALL_RADIUS: f32 = 0.03;

pub const TEXT_FLASH_TIME: f32 = 0.8;

const BALL_COLOR: egui::Color32 = egui::Color32::LIGHT_GREEN;

//The three possible colors for the Box to have, excluding the default WHITE.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BoxColor {
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "blue")]
    Blue,
}

impl From<BoxColor> for Color32 {
    fn from(value: BoxColor) -> Self {
        match value {
            BoxColor::Red => Color32::RED,
            BoxColor::Green => Color32::GREEN,
            BoxColor::Blue => Color32::BLUE,
        }
    }
}

//Keeps track of when a key is pressed and the time it has been since.
#[derive(new)]
pub struct TrackingWidget {
    ball_pos: Pos2,
    key_detected: bool,
    feedback_text: Option<String>,
    outline_color: Color32,
}

//This structure communicates with AppState in order to get the information needed.
#[derive(new)]
pub struct TrackingWidgetState {
    pub key_detected: bool,
    pub feedback_text: Option<String>,
    time_since: f32,
    pub outline_color: Color32,
}

impl TrackingWidgetState {
    pub fn blink(&mut self, feedback_text: Option<&str>, respond_color: Option<BoxColor>) {
        self.key_detected = true;
        self.outline_color = respond_color.map_or(FRAME_BORDER_COLOR, |c| c.into());
        self.feedback_text = feedback_text.map_or(None, |s| Some(s.to_string()));
    }

    //Keeps track of time since key detected and resets everything after the limit has been reached.
    pub fn update(&mut self, time: f32) {
        if self.key_detected {
            self.time_since += time;
            if self.time_since >= TEXT_FLASH_TIME {
                self.time_since = 0.0;
                self.key_detected = false;
                self.outline_color = FRAME_BORDER_COLOR;
            }
        }
    }
}

impl TrackingWidget {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
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
                egui::Stroke::new(FRAME_BORDER_WIDTH, self.outline_color),
            );

            painter.add(egui::Shape::Rect(frame_shape));

            // Draw the crosshair
            // The frame is guaranteed to be square
            let frame_width = rect.width();
            let crosshair_half_size = BALL_RADIUS * frame_width / 2.0;
            let center = rect.center();

            // Draw feedback text
            if self.key_detected {
                let text = self.feedback_text.clone().map_or("".to_string(), |s| s);
                let text_pos = Pos2::new(center.x, center.y * 0.05);
                let anchor = Align2::CENTER_TOP;
                let font_id = FontId::proportional(20.0);
                let text_color = Color32::WHITE;

                painter.text(text_pos, anchor, text, font_id, text_color);
            }

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
