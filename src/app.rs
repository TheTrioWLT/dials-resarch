use std::{collections::HashMap, sync::Mutex};

use eframe::{
    egui::{self, Frame, Key},
    emath::Vec2,
    epaint::Color32,
};

use crate::config::{ConfigAlarm, ConfigTrial};
use crate::{
    ball::Ball,
    config::InputMode,
    dial::Dial,
    dial_widget::{
        DialWidget, DIALS_HEIGHT_PERCENT, MAX_DIALS_WIDTH_PERCENT, MAX_DIAL_HEIGHT_PERCENT,
    },
    output::SessionOutput,
    tracking_widget::{TrackingWidget, TrackingWidgetState},
    DEFAULT_OUTPUT_PATH,
};

const UI_BACKGROUND_COLOR: Color32 = Color32::from_rgb(27, 27, 27);

// We don't really need extra indirection by Box-ing RunningState, we aren't moving a bunch
// of AppState's around all the time
#[allow(clippy::large_enum_variant)]
pub enum AppState {
    Running(RunningState),
    Done,
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Running(RunningState::new())
    }
}

pub struct RunningState {
    pub dial_rows: Vec<Vec<Dial>>,
    pub trials: Vec<ConfigTrial>,
    pub alarms: HashMap<String, ConfigAlarm>,
    pub ball: Ball,
    pub alarm_active: bool,
    pub current_trial_number: usize,
    /// The input axes as stored as [-1.0 to 1.0, -1.0 to 1.0]: [x, y]
    pub input_axes: Vec2,
    /// The input axes as stored as [0.0 to 1.0, 0.0 to 1.0]
    pub input_x: [f32; 2],
    pub input_y: [f32; 2],
    /// If a key was recently pressed which is to be interpreted as an alarm reaction
    pub pressed_key: Option<char>,
    pub last_keys: HashMap<Key, bool>,
    pub input_mode: InputMode,
    pub session_output: SessionOutput,
    pub num_alarms_done: usize,
    pub tracking_state: TrackingWidgetState,
}

impl RunningState {
    pub fn new() -> Self {
        Self {
            dial_rows: Vec::new(),
            trials: Vec::new(),
            alarms: HashMap::new(),
            ball: Ball::new(0.0, 0.0, crate::ball::BallVelocity::Slow),
            alarm_active: false,
            current_trial_number: 1,
            input_axes: Vec2::ZERO,
            input_x: [0.0, 0.0],
            input_y: [0.0, 0.0],
            pressed_key: None,
            last_keys: HashMap::new(),
            input_mode: InputMode::default(),
            session_output: SessionOutput::new(String::new()),
            num_alarms_done: 0,
            tracking_state: TrackingWidgetState::new(false, None, 0.0, Color32::WHITE),
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

        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = 72.0;
        }

        cc.egui_ctx.set_style(style);
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        let state = self.state_mutex.lock().unwrap();

        match &*state {
            AppState::Running(running_state) => {
                self.dial_ui(ctx, running_state);
                self.tracking_ui(ctx, running_state);
            }
            AppState::Done => {
                self.done_ui(ctx);
            }
        }
    }

    /// Draws the UI that shows when the trial is done
    fn done_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::none().fill(UI_BACKGROUND_COLOR))
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    // This has to be a *single* widget so that we don't have to do a bunch of math...
                    ui.label(format!(
                        "Trial Complete!\n\nTrial data saved to: {}",
                        DEFAULT_OUTPUT_PATH
                    ));
                });
            });
    }

    /// Draws the tracking task part of the UI
    fn tracking_ui(&mut self, ctx: &egui::Context, running_state: &RunningState) {
        let window_height = ctx.available_rect().height();

        egui::CentralPanel::default()
            .frame(Frame::none().fill(UI_BACKGROUND_COLOR))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(window_height * 0.025);
                    TrackingWidget::new(
                        running_state.ball.pos(),
                        running_state.tracking_state.key_detected,
                        running_state.tracking_state.feedback_text.clone(),
                        running_state.tracking_state.outline_color,
                    )
                    .show(ui);
                });
            });
    }

    /// Draws the dials part of the UI
    fn dial_ui(&mut self, ctx: &egui::Context, running_state: &RunningState) {
        let window_rect = ctx.available_rect();
        let window_height = window_rect.height();
        let window_width = window_rect.width();
        let bottom_panel_height = window_height * DIALS_HEIGHT_PERCENT;

        let num_rows_f = running_state.dial_rows.len() as f32;

        let x_spacing = window_width * 0.05;

        egui::TopBottomPanel::bottom("bottom_panel")
            .max_height(bottom_panel_height)
            .frame(Frame::none().fill(UI_BACKGROUND_COLOR))
            .show_separator_line(false)
            .show(ctx, |ui| {
                let max_num_dials = running_state
                    .dial_rows
                    .iter()
                    .map(|r| r.len())
                    .max()
                    .unwrap() as f32;

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
                    for row in &running_state.dial_rows {
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

        let mut state = self.state_mutex.lock().unwrap();

        match &mut *state {
            AppState::Running(state) => {
                let (mut input_x, mut input_y) = { (state.input_x, state.input_y) };

                let mut pressed_key = None;

                // Listen to events
                let events = ctx.input(|input| input.events.clone());

                for event in events {
                    if let egui::Event::Key {
                        key,
                        pressed,
                        modifiers: _,
                        repeat: _,
                    } = event
                    {
                        let last_pressed = { *state.last_keys.entry(key).or_insert(false) };
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
                                        k, Num1, '1', Num2, '2', Num3, '3', Num4, '4', Num5, '5',
                                        Num6, '6', Num7, '7', Num8, '8', Num9, '9', A, 'A', B, 'B',
                                        C, 'C', D, 'D', E, 'E', F, 'F', G, 'G', H, 'H', I, 'I', J,
                                        'J', K, 'K', L, 'L', M, 'M', N, 'N', O, 'O', P, 'P', Q,
                                        'Q', R, 'R', S, 'S', T, 'T', U, 'U', V, 'V', W, 'W', X,
                                        'X', Y, 'Y', Z, 'Z'
                                    );
                                }
                            }
                        }

                        state.last_keys.insert(key, pressed);
                    }
                }

                let input_axes = Vec2::new(input_x[0] - input_x[1], input_y[0] - input_y[1]);

                state.input_axes = input_axes;
                state.input_x = input_x;
                state.input_y = input_y;
                state.pressed_key = pressed_key;
            }
            AppState::Done => {}
        }

        // Ask for another repaint so that our app is continuously displayed
        ctx.request_repaint();
    }
}
