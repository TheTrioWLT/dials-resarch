use anyhow::Result;
use audio::AudioManager;
use dial::{Dial, DialRange};
use lazy_static::lazy_static;
use output::SessionOutput;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use app::{AppState, DialsApp};

use crate::{ball::Ball, dial::DialReaction};

mod app;
mod ball;
mod dial;
mod dial_widget;
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
                println!("failed to parse config file");
                println!("{}", e);
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

    let audio = Arc::new(AudioManager::new()?);

    // Maps alarm names to alarm structs
    let alarms: HashMap<&str, &config::Alarm> =
        config.alarms.iter().map(|d| (d.name.as_str(), d)).collect();

    for alarm in alarms.values() {
        if let Err(e) = audio.preload_file(&alarm.audio_path) {
            println!("failed to load audio file `{}`:", &alarm.audio_path);
            println!("{}", e);
            println!("does the file exist?");
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

    validate_config(&mut config);

    thread::spawn(move || model(&STATE));

    eframe::run_native(
        "Dials App",
        options,
        Box::new(move |cc| Box::new(DialsApp::new(cc, &STATE))),
    );
}

/// Our program's actual internal model, as opposted to the "view" which is our UI
fn model(state: &Mutex<AppState>) {
    let mut last_update = Instant::now();

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

            state.queued_alarms.extend(alarms);

            let input_axes = state.input_axes;

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
                println!("alarm `{alarm_name}` is missing");
                println!("available alarms are {alarm_names:?}");
                std::process::exit(1);
            }
        }
    }
    for alarm in &mut config.alarms {
        match alarm.clear_key.to_uppercase().to_string().chars().next() {
            Some(key) => alarm.clear_key = key,
            None => {
                println!("alarm `{}` is missing a clear key", alarm.name);
                std::process::exit(1);
            }
        }
    }
}
