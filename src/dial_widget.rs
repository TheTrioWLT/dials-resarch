use eframe::{
    egui,
    emath::{Pos2, Vec2},
    epaint::{CircleShape, Color32, Stroke},
};

use crate::dial::{DialRange, DIAL_MAX_VALUE};

pub const DIALS_MAX_WIDTH_PERCENT: f32 = 0.9;
pub const DIALS_HEIGHT_PERCENT: f32 = 0.3;
const NUM_DIAL_TICKS: u32 = 10;
const DIAL_TICK_RADIUS: f32 = 2.0;
const NUM_DIAL_BAR_MAX_VERTICES: u32 = 100;
const DIAL_BAR_WIDTH: f32 = 4.0;
const DIAL_NEEDLE_INSET: f32 = DIAL_BAR_WIDTH + 1.0;
const DIAL_ANGLE_OFFSET: f32 = -std::f32::consts::FRAC_PI_2;

const DIAL_TICK_COLOR: Color32 = Color32::WHITE;
const DIAL_BAR_COLOR: Color32 = Color32::YELLOW;
const DIAL_NEEDLE_COLOR: Color32 = Color32::YELLOW;

// const DIAL_BAR_TICK_VALUE: u32 = 1000;
const DIAL_NEEDLE_TICK_VALUE: f32 = 100.0;

const DIAL_NEEDLE_MAX: f32 = DIAL_NEEDLE_TICK_VALUE * NUM_DIAL_TICKS as f32;

pub struct DialWidget {
    value: f32,
    radius: f32,
    in_range: DialRange,
}

impl DialWidget {
    pub fn new(value: f32, radius: f32, in_range: DialRange) -> Self {
        Self {
            value,
            radius,
            in_range,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = Vec2::splat(self.radius);
        let (rect, mut response) =
            ui.allocate_exact_size(desired_size, egui::Sense::focusable_noninteractive());

        // Only draw if we need to
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            let center = rect.center();
            let radius = rect.width() / 2.0;

            // Draw the ticks
            let tick_dist = std::f32::consts::TAU / NUM_DIAL_TICKS as f32;
            for i in 0..NUM_DIAL_TICKS {
                let angle = (i as f32 * tick_dist) + DIAL_ANGLE_OFFSET;
                let x = radius * f32::cos(angle);
                let y = radius * f32::sin(angle);
                let position = Pos2::new(x + center.x, y + center.y);

                painter.add(egui::Shape::Circle(CircleShape::filled(
                    position,
                    DIAL_TICK_RADIUS,
                    DIAL_TICK_COLOR,
                )));
            }

            // Draw the "bar"
            let bar_angle_percent = self.value / DIAL_MAX_VALUE;
            let num_dial_bar_vertices =
                (bar_angle_percent * NUM_DIAL_BAR_MAX_VERTICES as f32) as u32;
            let bar_dist =
                (bar_angle_percent * std::f32::consts::TAU) / num_dial_bar_vertices as f32;
            // Change this initial position when DIAL_ANGLE_OFFSET changes
            let mut last_vertex_pos = Pos2::new(center.x, center.y - radius);

            for i in 1..num_dial_bar_vertices + 1 {
                let angle = (i as f32 * bar_dist) + DIAL_ANGLE_OFFSET;
                let x = radius * f32::cos(angle);
                let y = radius * f32::sin(angle);
                let position = Pos2::new(x + center.x, y + center.y);

                painter.add(egui::Shape::LineSegment {
                    points: [last_vertex_pos, position],
                    stroke: Stroke::new(DIAL_BAR_WIDTH, DIAL_BAR_COLOR),
                });

                last_vertex_pos = position;
            }

            #[cfg(feature = "debugging")]
            {
                // If in debugging mode, this will draw the dial's in-range
                let start_radians = (self.in_range.start / DIAL_MAX_VALUE) * std::f32::consts::TAU
                    + DIAL_ANGLE_OFFSET;
                let end_radians = (self.in_range.end / DIAL_MAX_VALUE) * std::f32::consts::TAU
                    + DIAL_ANGLE_OFFSET;
                let radians_dist = (end_radians - start_radians) / 100.0;
                let start_x = (radius + DIAL_BAR_WIDTH * 1.0) * f32::cos(start_radians);
                let start_y = (radius + DIAL_BAR_WIDTH * 1.0) * f32::sin(start_radians);

                let mut last_vertex_pos = Pos2::new(center.x + start_x, center.y + start_y);

                for i in 1..101 {
                    let angle = (i as f32 * radians_dist) + start_radians;
                    let x = (radius + DIAL_BAR_WIDTH * 1.0) * f32::cos(angle);
                    let y = (radius + DIAL_BAR_WIDTH * 1.0) * f32::sin(angle);
                    let position = Pos2::new(x + center.x, y + center.y);

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
            let needle_inset_radius = radius - DIAL_NEEDLE_INSET;
            let end_position = Pos2::new(
                center.x + needle_inset_radius * f32::cos(needle_angle_radians),
                center.y + needle_inset_radius * f32::sin(needle_angle_radians),
            );

            painter.add(egui::Shape::LineSegment {
                points: [center, end_position],
                stroke: Stroke::new(2.0, DIAL_NEEDLE_COLOR),
            });
        }

        response.mark_changed();

        response
    }
}
