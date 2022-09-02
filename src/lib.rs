use anyhow::Result;
use audio::AudioManager;
use ball::Ball;
use dial::{Dial, DialRange};
use eframe::emath::Vec2;
use lazy_static::lazy_static;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use app::{AppState, DialsApp};

use crate::dial::DialReaction;

mod app;
mod ball;
mod dial;
mod dial_widget;
mod tracking_widget;

pub mod audio;
pub mod config;

pub const DEFAULT_INPUT_PATH: &str = "./config.toml";

lazy_static! {
    static ref STATE: Mutex<AppState> = Mutex::new(AppState {
        dials: Vec::new(),
        ball: Ball::new(),
        input_axes: Vec2::ZERO,
        input_x: [0.0, 0.0],
        input_y: [0.0, 0.0],
        pressed_key: None,
        queued_alarms: VecDeque::new()
    });
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

    let dials: Vec<_> = config
        .dials
        .iter()
        .enumerate()
        .map(|(id, dial)| {
            let alarm = alarms[dial.alarm.as_str()];
            Dial::new(
                id,
                dial.rate,
                DialRange::new(dial.start, dial.end),
                alarm,
                Arc::clone(&audio),
            )
        })
        .collect();

    {
        let mut state = STATE.lock().unwrap();

        state.dials = dials;
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

    loop {
        thread::sleep(Duration::from_millis(2));

        let delta_time = last_update.elapsed().as_secs_f32();

        if let Ok(mut state) = state.lock() {
            let mut alarms = Vec::new();

            for dial in state.dials.iter_mut() {
                if let Some(alarm) = dial.update(delta_time) {
                    alarms.push(alarm);
                }
            }

            state.queued_alarms.extend(alarms);

            let input_axes = state.input_axes;

            state.ball.update(input_axes, delta_time);

            if let Some(key) = state.pressed_key {
                if let Some(alarm) = state.queued_alarms.pop_front() {
                    let millis = alarm.time.elapsed().as_millis() as u32;

                    let reaction =
                        DialReaction::new(alarm.dial_id, millis, alarm.correct_key == key, key);

                    state.dials[alarm.dial_id].reset();

                    println!("{reaction:?}");
                }
            }
        }

        last_update = Instant::now();
    }
}

/// Validates a config file, or exits the program with an error printed to the command line on how
/// to fix the validation
fn validate_config(config: &mut config::Config) {
    if let Some(active) = &config.active_ball {
        let ball_names: Vec<_> = config.balls.iter().map(|b| &b.name).collect();
        if !ball_names.contains(&active) {
            println!("active ball `{active}` is missing");
            println!("available balls are {ball_names:?}");
            std::process::exit(1);
        }
    }

    let alarm_names: Vec<_> = config.alarms.iter().map(|b| &b.name).collect();
    for dial in &config.dials {
        let alarm_name = &dial.alarm;
        if !alarm_names.contains(&alarm_name) {
            println!("alarm `{alarm_name}` is missing");
            println!("available alarms are {alarm_names:?}");
            std::process::exit(1);
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
