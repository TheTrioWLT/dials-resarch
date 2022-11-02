use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use eframe::{
    egui::{self, Frame, Key},
    emath::Vec2,
    epaint::Color32,
};

use crate::{
    ball::Ball,
    config::InputMode,
    dial::{Dial, TriggeredAlarm},
    dial_widget::{
        DialWidget, DIALS_HEIGHT_PERCENT, MAX_DIALS_WIDTH_PERCENT, MAX_DIAL_HEIGHT_PERCENT,
    },
    output::SessionOutput,
    tracking_widget::TrackingWidget,
};

pub struct AppState {
    pub dial_rows: Vec<Vec<Dial>>,
    pub ball: Ball,
    pub input_axes: Vec2,
    pub input_x: [f32; 2],
    pub input_y: [f32; 2],
    pub pressed_key: Option<char>,
    pub queued_alarms: VecDeque<TriggeredAlarm>,
    pub last_keys: HashMap<Key, bool>,
    pub input_mode: InputMode,
    pub session_output: SessionOutput,
    pub num_alarms_done: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            dial_rows: Vec::new(),
            ball: Ball::new(0.0, 0.0, crate::ball::BallVelocity::Slow),
            input_axes: Vec2::ZERO,
            input_x: [0.0, 0.0],
            input_y: [0.0, 0.0],
            pressed_key: None,
            queued_alarms: VecDeque::new(),
            last_keys: HashMap::new(),
            input_mode: InputMode::default(),
            session_output: SessionOutput::new(String::new()),
            num_alarms_done: 0,
        }
    }
}

pub struct DialsApp {
    state_mutex: &'static Mutex<AppState>,
}

impl DialsApp {
    pub fn new(cc: &eframe::CreationContext, state_mutex: &'static Mutex<AppState>) -> Self {
        DialsApp::style(cc);

        Self { state_mutex }
    }

    fn style(cc: &eframe::CreationContext) {
        let mut style = egui::style::Style::default();

        style.visuals = egui::style::Visuals::dark();

        cc.egui_ctx.set_style(style);
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        self.dial_ui(ctx);
        self.tracking_ui(ctx);
    }

    /// Draws the tracking task part of the UI
    fn tracking_ui(&mut self, ctx: &egui::Context) {
        let window_height = ctx.available_rect().height();

        let state = self.state_mutex.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(window_height * 0.025);
                TrackingWidget::new(state.ball.pos()).show(ui);
            });
        });
    }

    /// Draws the dials part of the UI
    fn dial_ui(&mut self, ctx: &egui::Context) {
        let window_rect = ctx.available_rect();
        let window_height = window_rect.height();
        let window_width = window_rect.width();
        let bottom_panel_height = window_height * DIALS_HEIGHT_PERCENT;

        let state = self.state_mutex.lock().unwrap();

        let num_rows_f = state.dial_rows.len() as f32;

        let x_spacing = window_width * 0.05;

        egui::TopBottomPanel::bottom("bottom_panel")
            .max_height(bottom_panel_height)
            .frame(Frame::none().fill(Color32::from_rgb(27, 27, 27)))
            .show(ctx, |ui| {
                let max_num_dials = state.dial_rows.iter().map(|r| r.len()).max().unwrap() as f32;

                // We want there to be at least a little space in between dial rows
                let min_y_spacing = bottom_panel_height * 0.1;

                // These are all of our nice constraints for how large dials can be:
                // Based on the width of the window
                // Based on the height of the window
                // Based on the max that still allows for the minimum spacing
                let width_dial_max_radius =
                    (window_width * MAX_DIALS_WIDTH_PERCENT) / (max_num_dials * 2.0);
                let height_dial_max_radius = bottom_panel_height * MAX_DIAL_HEIGHT_PERCENT;
                let spacing_dial_max_radius =
                    (bottom_panel_height - ((num_rows_f + 1.0) * min_y_spacing)) / num_rows_f;

                // The dial radius is the minimum of each maximum
                let dial_radius = height_dial_max_radius
                    .min(width_dial_max_radius)
                    .min(spacing_dial_max_radius);

                // Calculate our spacing values
                let dials_total_height = dial_radius * num_rows_f;
                let total_y_spacing = bottom_panel_height - dials_total_height;
                let y_spacing = total_y_spacing / (num_rows_f + 1.0);

                ui.add_space(y_spacing);

                ui.vertical_centered_justified(|ui| {
                    for row in &state.dial_rows {
                        let num_dials = row.len();

                        let items_width =
                            num_dials as f32 * dial_radius + ((num_dials - 1) as f32 * x_spacing);

                        // Required to make these widgets centered
                        ui.set_max_width(items_width);

                        ui.horizontal(|ui| {
                            ui.set_height(dial_radius);

                            ui.spacing_mut().item_spacing.x = x_spacing;

                            for dial in row.iter() {
                                DialWidget::new(dial.value(), dial_radius, dial.in_range())
                                    .show(ui);
                            }
                        });

                        ui.add_space(y_spacing);
                    }
                });
            });
    }
}

/// Map a key press `k` to the `char` it corresponds with
macro_rules! key_to_char {
    ($k:expr, $($case:path, $lit:literal),+) => {
        match $k {
            $($case => Some($lit),)+
            _ => None,
        }
    };
}

impl eframe::App for DialsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Draw the UI
        self.ui(ctx);

        let (mut input_x, mut input_y) = {
            let state = self.state_mutex.lock().unwrap();

            (state.input_x, state.input_y)
        };

        let mut pressed_key = None;

        // Listen to events
        let events = ctx.input().events.clone();

        for event in events {
            if let egui::Event::Key {
                key,
                pressed,
                modifiers: _,
            } = event
            {
                let last_pressed = {
                    let mut state = self.state_mutex.lock().unwrap();

                    *state.last_keys.entry(key).or_insert(false)
                };
                let value = if pressed { 1.0 } else { 0.0 };
                // true if the this key changed from last time
                // (was just pressed or released since the last frame)
                let key_changed = pressed != last_pressed;

                match key {
                    Key::ArrowUp => input_y[0] = value,
                    Key::ArrowDown => input_y[1] = value,
                    Key::ArrowRight => input_x[0] = value,
                    Key::ArrowLeft => input_x[1] = value,
                    k => {
                        use egui::Key::*;

                        if key_changed && pressed {
                            pressed_key = key_to_char!(
                                k, Num1, '1', Num2, '2', Num3, '3', Num4, '4', Num5, '5', Num6,
                                '6', Num7, '7', Num8, '8', Num9, '9', A, 'A', B, 'B', C, 'C', D,
                                'D', E, 'E', F, 'F', G, 'G', H, 'H', I, 'I', J, 'J', K, 'K', L,
                                'L', M, 'M', N, 'N', O, 'O', P, 'P', Q, 'Q', R, 'R', S, 'S', T,
                                'T', U, 'U', V, 'V', W, 'W', X, 'X', Y, 'Y', Z, 'Z'
                            );
                        }
                    }
                }
                {
                    let mut state = self.state_mutex.lock().unwrap();
                    state.last_keys.insert(key, pressed);
                }
            }
        }

        let input_axes = Vec2::new(input_x[0] - input_x[1], input_y[0] - input_y[1]);

        {
            let mut state = self.state_mutex.lock().unwrap();

            state.input_axes = input_axes;
            state.input_x = input_x;
            state.input_y = input_y;
            state.pressed_key = pressed_key;
        }

        // Ask for another repaint so that our app is continously displayed
        ctx.request_repaint();
    }
}
