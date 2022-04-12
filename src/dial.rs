use egui::{epaint::CircleShape, Color32, Pos2, Stroke};

pub const DIAL_Y_OFFSET_PERCENT: f32 = 0.03;
pub const DIAL_HEIGHT_PERCENT: f32 = 0.3;
pub const DIALS_MAX_WIDTH_PERCENT: f32 = 0.6;

const NUM_DIAL_TICKS: u32 = 10;
const DIAL_TICK_RADIUS: f32 = 2.0;
const DIAL_TICK_INSET: f32 = DIAL_TICK_RADIUS * 2.0;

pub struct DialDrawData {
    pub y_offset: f32,
    pub radius: f32,
    pub dial_width_percent: f32,
    pub window_width: f32,
    pub window_left_bottom: Pos2,
}

pub struct Dial {
    value: f32,
    dial_num: u32,
}

impl Dial {
    pub fn new(dial_num: u32) -> Self {
        Self {
            value: 0.0,
            dial_num,
        }
    }

    pub fn draw(&self, painter: &egui::Painter, draw_data: &DialDrawData) {
        let dial_pos_x =
            self.dial_num as f32 * draw_data.dial_width_percent * draw_data.window_width;
        let dial_center = draw_data.window_left_bottom
            + Pos2::new(dial_pos_x, -draw_data.radius - draw_data.y_offset).to_vec2();

        painter.add(egui::Shape::Circle(CircleShape::stroke(
            dial_center,
            draw_data.radius,
            Stroke::new(2.0, Color32::LIGHT_GREEN),
        )));

        let tick_inset_radius = draw_data.radius - DIAL_TICK_INSET;
        let dist = std::f32::consts::TAU / NUM_DIAL_TICKS as f32;

        for i in 0..NUM_DIAL_TICKS {
            let angle = i as f32 * dist;
            let x = tick_inset_radius * f32::cos(angle);
            let y = tick_inset_radius * f32::sin(angle);
            let position = Pos2::new(x + dial_center.x, y + dial_center.y);

            painter.add(egui::Shape::Circle(CircleShape::filled(
                position,
                DIAL_TICK_RADIUS,
                Color32::LIGHT_YELLOW,
            )));
        }

        let dial_angle_radians = (self.value / 100.0) * std::f32::consts::TAU;
        let end_position = Pos2::new(
            dial_center.x + tick_inset_radius * f32::cos(dial_angle_radians),
            dial_center.y + tick_inset_radius * f32::sin(dial_angle_radians),
        );

        painter.add(egui::Shape::LineSegment {
            points: [dial_center, end_position],
            stroke: Stroke::new(2.0, Color32::WHITE),
        });
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value % std::f32::consts::TAU;
    }

    pub fn increment_value(&mut self, increment: f32) {
        self.value = (self.value + increment) % std::f32::consts::TAU;
    }
}
