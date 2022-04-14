use egui::{epaint::CircleShape, Color32, Pos2, Stroke};

pub const DIAL_Y_OFFSET_PERCENT: f32 = 0.03;
pub const DIAL_HEIGHT_PERCENT: f32 = 0.3;
pub const DIALS_MAX_WIDTH_PERCENT: f32 = 0.6;

const NUM_DIAL_TICKS: u32 = 10;
const DIAL_TICK_RADIUS: f32 = 2.0;
const NUM_DIAL_BAR_MAX_VERTICES: u32 = 100;
const DIAL_BAR_WIDTH: f32 = 4.0;
const DIAL_NEEDLE_INSET: f32 = DIAL_BAR_WIDTH + 1.0;
const DIAL_ANGLE_OFFSET: f32 = -std::f32::consts::FRAC_PI_2;

const DIAL_TICK_COLOR: Color32 = Color32::WHITE;
const DIAL_BAR_COLOR: Color32 = Color32::YELLOW;
const DIAL_NEEDLE_COLOR: Color32 = Color32::YELLOW;

const DIAL_MAX_VALUE: u32 = 10000;
// const DIAL_BAR_TICK_VALUE: u32 = 1000;
const DIAL_NEEDLE_TICK_VALUE: u32 = 100;

const DIAL_NEEDLE_MAX: u32 = DIAL_NEEDLE_TICK_VALUE * NUM_DIAL_TICKS;

pub struct DialDrawData {
    pub y_offset: f32,
    pub radius: f32,
    pub dial_width_percent: f32,
    pub window_width: f32,
    pub window_left_bottom: Pos2,
}

pub struct Dial {
    value: u32,
    dial_num: u32,
}

impl Dial {
    pub fn new(dial_num: u32) -> Self {
        Self {
            value: 0,
            dial_num,
        }
    }

    pub fn draw(&self, painter: &egui::Painter, draw_data: &DialDrawData) {
        let dial_pos_x =
            self.dial_num as f32 * draw_data.dial_width_percent * draw_data.window_width;
        let dial_center = draw_data.window_left_bottom
            + Pos2::new(dial_pos_x, -draw_data.radius - draw_data.y_offset).to_vec2();

        // Draw the ticks
        let tick_dist = std::f32::consts::TAU / NUM_DIAL_TICKS as f32;
        for i in 0..NUM_DIAL_TICKS {
            let angle = (i as f32 * tick_dist) + DIAL_ANGLE_OFFSET;
            let x = draw_data.radius * f32::cos(angle);
            let y = draw_data.radius * f32::sin(angle);
            let position = Pos2::new(x + dial_center.x, y + dial_center.y);

            painter.add(egui::Shape::Circle(CircleShape::filled(
                position,
                DIAL_TICK_RADIUS,
                DIAL_TICK_COLOR,
            )));
        }

        // Draw the "bar"
        let bar_angle_percent = self.value as f32 / DIAL_MAX_VALUE as f32;
        let num_dial_bar_vertices = (bar_angle_percent * NUM_DIAL_BAR_MAX_VERTICES as f32) as u32;
        let bar_dist = (bar_angle_percent * std::f32::consts::TAU) / num_dial_bar_vertices as f32;
        // Change this initial position when DIAL_ANGLE_OFFSET changes
        let mut last_vertex_pos = Pos2::new(dial_center.x, dial_center.y - draw_data.radius);

        for i in 1..num_dial_bar_vertices + 1 {
            let angle = (i as f32 * bar_dist) + DIAL_ANGLE_OFFSET;
            let x = draw_data.radius * f32::cos(angle);
            let y = draw_data.radius * f32::sin(angle);
            let position = Pos2::new(x + dial_center.x, y + dial_center.y);

            painter.add(egui::Shape::LineSegment { points: [last_vertex_pos, position], stroke: Stroke::new(DIAL_BAR_WIDTH, DIAL_BAR_COLOR) });

            last_vertex_pos = position;
        }

        // Draw the needle
        let needle_angle_radians = (((self.value % (DIAL_NEEDLE_MAX)) as f32 / DIAL_NEEDLE_MAX as f32) * std::f32::consts::TAU) + DIAL_ANGLE_OFFSET;
        let needle_inset_radius = draw_data.radius - DIAL_NEEDLE_INSET;
        let end_position = Pos2::new(
            dial_center.x + needle_inset_radius * f32::cos(needle_angle_radians),
            dial_center.y + needle_inset_radius * f32::sin(needle_angle_radians),
        );

        painter.add(egui::Shape::LineSegment {
            points: [dial_center, end_position],
            stroke: Stroke::new(2.0, DIAL_NEEDLE_COLOR),
        });
    }

    pub fn set_value(&mut self, value: u32) {
        self.value = value % DIAL_MAX_VALUE;
    }

    pub fn increment_value(&mut self, increment: u32) {
        self.value = (self.value + increment) % DIAL_MAX_VALUE;
        dbg!(self.value);
    }
}
