use serde::{Deserialize, Serialize};
use std::{thread, time::Instant};

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

const DIAL_MAX_VALUE: f32 = 10000.0;
// const DIAL_BAR_TICK_VALUE: u32 = 1000;
const DIAL_NEEDLE_TICK_VALUE: f32 = 100.0;

const DIAL_NEEDLE_MAX: f32 = DIAL_NEEDLE_TICK_VALUE * NUM_DIAL_TICKS as f32;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct DialRange {
    pub start: f32,
    pub end: f32,
}

impl DialRange {
    pub fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, value: f32) -> bool {
        value <= self.end && value >= self.start
    }

    pub fn random_in(&self) -> f32 {
        self.start + (self.end - self.start) * rand::random::<f32>()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DialDrawData {
    pub y_offset: f32,
    pub radius: f32,
    pub dial_width_percent: f32,
    pub window_width: f32,
    pub window_left_bottom: Pos2,
}

#[derive(Debug, Copy, Clone)]
pub struct DialReaction {
    pub dial_id: usize,
    pub millis: u32,
    pub correct_key: bool,
    pub key: char,
}

#[derive(Debug, Copy, Clone)]
pub struct DialAlarm {
    pub dial_id: usize,
    pub time: Instant,
    pub correct_key: char,
}

impl DialAlarm {
    pub fn new(dial_id: usize, time: Instant, correct_key: char) -> Self {
        Self {
            dial_id,
            time,
            correct_key,
        }
    }
}

impl DialReaction {
    pub fn new(dial_id: usize, millis: u32, correct_key: bool, key: char) -> Self {
        Self {
            dial_id,
            millis,
            correct_key,
            key,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Dial {
    value: f32,
    dial_id: usize,
    rate: f32,
    in_range: DialRange,
    key: char,
    alarm_fired: bool,
}

impl Dial {
    pub fn new(dial_id: usize, rate: f32, in_range: DialRange, key: char) -> Self {
        let mut dial = Self {
            value: 0.0,
            dial_id,
            rate,
            in_range,
            key,
            alarm_fired: false,
        };

        // Immediately "reset"
        dial.reset();

        dial
    }

    pub fn reset(&mut self) {
        let reset_value = self.in_range.random_in();

        let new_rate = if rand::random::<bool>() {
            self.rate
        } else {
            -self.rate
        };

        self.value = reset_value;
        self.rate = new_rate;
        self.alarm_fired = false;
    }

    /// Updates the dial using the amount of time that has passed since the last update
    /// A DialReaction data structure is returned if this dial has gone out of range.
    pub fn update(&mut self, delta_time: f32) -> Option<DialAlarm> {
        // Increment the value using the rate and the delta time
        self.increment_value(delta_time * self.rate);

        if !self.alarm_fired && !self.in_range.contains(self.value) {
            self.on_out_of_range();

            let dial_alarm = DialAlarm::new(self.dial_id, Instant::now(), self.key);

            Some(dial_alarm)
        } else {
            None
        }
    }

    pub fn draw(&mut self, painter: &egui::Painter, draw_data: &DialDrawData) {
        let dial_pos_x =
            (self.dial_id + 1) as f32 * draw_data.dial_width_percent * draw_data.window_width;
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
        let bar_angle_percent = self.value / DIAL_MAX_VALUE;
        let num_dial_bar_vertices = (bar_angle_percent * NUM_DIAL_BAR_MAX_VERTICES as f32) as u32;
        let bar_dist = (bar_angle_percent * std::f32::consts::TAU) / num_dial_bar_vertices as f32;
        // Change this initial position when DIAL_ANGLE_OFFSET changes
        let mut last_vertex_pos = Pos2::new(dial_center.x, dial_center.y - draw_data.radius);

        for i in 1..num_dial_bar_vertices + 1 {
            let angle = (i as f32 * bar_dist) + DIAL_ANGLE_OFFSET;
            let x = draw_data.radius * f32::cos(angle);
            let y = draw_data.radius * f32::sin(angle);
            let position = Pos2::new(x + dial_center.x, y + dial_center.y);

            painter.add(egui::Shape::LineSegment {
                points: [last_vertex_pos, position],
                stroke: Stroke::new(DIAL_BAR_WIDTH, DIAL_BAR_COLOR),
            });

            last_vertex_pos = position;
        }

        #[cfg(feature = "debugging")]
        {
            // If in debugging mode, this will draw the dial's in-range
            let start_radians =
                (self.in_range.start / DIAL_MAX_VALUE) * std::f32::consts::TAU + DIAL_ANGLE_OFFSET;
            let end_radians =
                (self.in_range.end / DIAL_MAX_VALUE) * std::f32::consts::TAU + DIAL_ANGLE_OFFSET;
            let radians_dist = (end_radians - start_radians) / 100.0;
            let start_x = (draw_data.radius + DIAL_BAR_WIDTH * 1.0) * f32::cos(start_radians);
            let start_y = (draw_data.radius + DIAL_BAR_WIDTH * 1.0) * f32::sin(start_radians);

            let mut last_vertex_pos = Pos2::new(dial_center.x + start_x, dial_center.y + start_y);

            for i in 1..101 {
                let angle = (i as f32 * radians_dist) + start_radians;
                let x = (draw_data.radius + DIAL_BAR_WIDTH * 1.0) * f32::cos(angle);
                let y = (draw_data.radius + DIAL_BAR_WIDTH * 1.0) * f32::sin(angle);
                let position = Pos2::new(x + dial_center.x, y + dial_center.y);

                painter.add(egui::Shape::LineSegment {
                    points: [last_vertex_pos, position],
                    stroke: Stroke::new(DIAL_BAR_WIDTH, Color32::LIGHT_GREEN),
                });

                last_vertex_pos = position;
            }
        }

        // Draw the needle
        let needle_angle_radians = (((self.value % (DIAL_NEEDLE_MAX as f32)) as f32
            / DIAL_NEEDLE_MAX as f32)
            * std::f32::consts::TAU)
            + DIAL_ANGLE_OFFSET;
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

    fn on_out_of_range(&mut self) {
        thread::spawn(|| crate::audio::play().unwrap());
        self.alarm_fired = true;
    }

    fn increment_value(&mut self, increment: f32) {
        self.value = (self.value + increment) % DIAL_MAX_VALUE;
    }
}
