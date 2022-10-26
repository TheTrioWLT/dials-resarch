use anyhow::Result;

use audio::AudioManager;
use dial::{Dial, DialRange};
use eframe::epaint::Vec2;
use lazy_static::lazy_static;
use output::SessionOutput;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use app::{AppState, DialsApp};

use crate::error_popup::ErrorPopup;
use crate::{ball::Ball, dial::DialReaction};
use gilrs::{Event, Gilrs};

mod app;
mod ball;
mod dial;
mod dial_widget;
mod error_popup;
mod output;
mod tracking_widget;

pub mod audio;
pub mod config;

pub const DEFAULT_INPUT_PATH: &str = "./config.toml";
pub const DEFAULT_OUTPUT_PATH: &str = "./trial.csv";

lazy_static! {
    static ref STATE: Mutex<AppState> = Mutex::new(AppState::new());
}

pub fn run() -> Result<()> {
    let options = eframe::NativeOptions {
        transparent: true,
        vsync: true,
        maximized: true,
        ..eframe::NativeOptions::default()
    };

    let mut config = match std::fs::read_to_string(DEFAULT_INPUT_PATH) {
        Ok(toml) => match toml::from_str(&toml) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to parse configuration file: {}", e);

                let popup = ErrorPopup::new(
                    "Configuration Error",
                    "Failed to parse configuration file",
                    format!("{}", e),
                );
                popup.show();

                std::process::exit(1);
            }
        },
        Err(_) => {
            // Write out default config if none existed before
            let config = config::Config::default();
            let toml = toml::to_string(&config)?;
            std::fs::write(DEFAULT_INPUT_PATH, &toml)?;

            config
        }
    };

    validate_config(&mut config);

    let audio = Arc::new(AudioManager::new()?);

    // Maps alarm names to alarm structs
    let alarms: HashMap<&str, &config::Alarm> =
        config.alarms.iter().map(|d| (d.name.as_str(), d)).collect();

    for alarm in alarms.values() {
        if let Err(e) = audio.preload_file(&alarm.audio_path) {
            eprintln!("failed to load audio file `{}`: {}", &alarm.audio_path, e);
            eprintln!("Does the file exist?");

            let message = format!("Failed to load {}\n{}", &alarm.audio_path, e);
            let popup = ErrorPopup::new("Audio Load Error", "Failed to load audio file", message);
            popup.show();

            std::process::exit(1);
        }
    }

    let dial_rows: Vec<_> = config
        .dial_rows
        .iter()
        .enumerate()
        .map(|(row_id, row)| {
            row.dials
                .iter()
                .enumerate()
                .map(|(id, dial)| {
                    let alarm = alarms[dial.alarm.as_str()];
                    Dial::new(
                        row_id,
                        id,
                        DialRange::new(dial.start, dial.end),
                        alarm,
                        Arc::clone(&audio),
                        dial.alarm_time,
                    )
                })
                .collect()
        })
        .collect();

    {
        let mut state = STATE.lock().unwrap();

        state.input_mode = config.input_mode;
        state.dial_rows = dial_rows;
        state.ball = Ball::new(
            config.ball.random_direction_change_time_min,
            config.ball.random_direction_change_time_max,
            config.ball.velocity_meter,
        );
        state.session_output = SessionOutput::new(
            config
                .output_data_path
                .clone()
                .unwrap_or_else(|| String::from(DEFAULT_OUTPUT_PATH)),
        );
    }

    thread::spawn(move || model(&STATE));

    eframe::run_native(
        "Dials App",
        options,
        Box::new(move |cc| Box::new(DialsApp::new(cc, &STATE))),
    );
}

/// Our program's actual internal model, as opposted to the "view" which is our UI
fn model(state: &Mutex<AppState>) {
    let mut gilrs = Gilrs::new().unwrap();

    let mut last_update = Instant::now();

    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?} ", gamepad.name(), gamepad.power_info());
    }

    let mut joystick_input_axes = Vec2::default();
    let total_num_alarms = {
        let state = state.lock().expect("This shouldn't fail silently");

        state.dial_rows.iter().map(|r| r.len()).sum()
    };

    loop {
        thread::sleep(Duration::from_millis(2));

        let delta_time = last_update.elapsed().as_secs_f32();

        if let Ok(mut state) = state.lock() {
            let mut alarms = Vec::new();

            for row in state.dial_rows.iter_mut() {
                for dial in row.iter_mut() {
                    if let Some(alarm) = dial.update(delta_time) {
                        alarms.push(alarm);
                    }
                }
            }

            while let Some(Event {event, .. }) = gilrs.next_event() {
                if let gilrs::ev::EventType::AxisChanged(axis, amount, _) = event {
                    match axis {
                        gilrs::ev::Axis::LeftStickX => {
                            joystick_input_axes[1] = -amount;
                        }
                        gilrs::ev::Axis::LeftStickY => {
                            joystick_input_axes[0] = -amount;
                        }
                        _ => {}
                    }
                }
            }

            state.queued_alarms.extend(alarms);

            let input_axes = match state.input_mode {
                config::InputMode::Joystick => joystick_input_axes,
                config::InputMode::Keyboard => state.input_axes,
            };

            state.ball.update(input_axes, delta_time);

            if let Some(key) = state.pressed_key {
                if let Some(alarm) = state.queued_alarms.pop_front() {
                    let millis = alarm.time.elapsed().as_millis() as u32;

                    let reaction = DialReaction::new(
                        alarm.dial_id,
                        millis,
                        alarm.correct_key == key,
                        key,
                        state.ball.current_rms_error(),
                    );

                    state.dial_rows[alarm.row_id][alarm.dial_id].reset();

                    state.session_output.add_reaction(reaction);

                    state.num_alarms_done += 1;

                    if state.num_alarms_done == total_num_alarms {
                        state.session_output.write_to_file();
                        log::info!(
                            "wrote session output to file: {}",
                            state.session_output.output_path
                        );
                    }
                }

                state.pressed_key = None;
            }
        }

        last_update = Instant::now();
    }
}

/// Validates a config file, or exits the program with an error printed to the command line on how
/// to fix the validation
fn validate_config(config: &mut config::Config) {
    let alarm_names: Vec<_> = config.alarms.iter().map(|b| &b.name).collect();
    for row in &config.dial_rows {
        for dial in &row.dials {
            let alarm_name = &dial.alarm;
            if !alarm_names.contains(&alarm_name) {
                eprintln!("Alarm `{alarm_name}` is missing!");
                eprintln!("Available alarms are {alarm_names:?}");

                let message = format!(
                    "Alarm `{alarm_name}` is missing!\nAvailable alarms are: {alarm_names:?}"
                );
                let popup =
                    ErrorPopup::new("Configuration Error", "Invalid configuration", message);
                popup.show();

                std::process::exit(1);
            }
        }
    }
}
