use anyhow::Result;

use audio::AudioManager;
use dial::{Dial, DialRange};
use eframe::epaint::Vec2;
use lazy_static::lazy_static;
use output::SessionOutput;
use pasts::Loop;
use rodio::cpal::Data;
use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    task::Poll::{self, Pending, Ready},
    thread,
    time::{Duration, Instant},
};

use crate::{ball::Ball, dial::DialReaction};
use app::{AppState, DialsApp};
use stick::{Controller, Event, Listener};

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

//Following stick's hello world guide
type Exit = usize;

struct State {
    listener: Listener,
    controllers: Vec<Controller>,
    input: Vec2,
    sender: Sender<Vec2>,
}

impl State {
    fn new(sender: Sender<Vec2>) -> Self {
        Self {
            listener: Listener::default(),
            controllers: Vec::new(),
            input: Vec2::ZERO,
            sender,
        }
    }

    fn connect(&mut self, controller: Controller) -> Poll<Exit> {
        println!(
            "Connected p{}, id: {:016X}, name: {}",
            self.controllers.len() + 1,
            controller.id(),
            controller.name()
        );
        self.controllers.push(controller);
        Pending
    }

    fn event(&mut self, id: usize, event: Event) -> Poll<Exit> {
        let player = id + 1;
        println!("p{}: {}", player, event);
        match event {
            Event::Disconnect => {
                self.controllers.swap_remove(id);
            }
            Event::MenuR(true) => return Ready(player),
            Event::JoyX(pressed) => {
                self.input.x = pressed as f32;
            }
            Event::JoyY(pressed) => {
                self.input.y = -pressed as f32;
            }
            _ => {}
        }
        self.sender.send(self.input).unwrap();

        Pending
    }
}

async fn event_loop<'a>(mut state: State) {
    let player_id = Loop::new(&mut state)
        .when(|s| &mut s.listener, State::connect)
        .poll(|s| &mut s.controllers, State::event)
        .await;

    println!("p{} ended the session", player_id);
}

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

    validate_config(&mut config);

    let (sender, receiver) = channel();

    thread::spawn(move || {
        let state = State::new(sender);
        pasts::block_on(event_loop(state));

        println!("p{:?} ended the session", ":(");
    });

    {
        let mut state = STATE.lock().unwrap();
        state.reciever = Some(receiver);
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
    let mut last_update = Instant::now();

    let total_num_alarms = {
        let state = state.lock().unwrap();

        state.dial_rows.iter().map(|r| r.len()).sum()
    };

    let mut joystick_input_axes = Vec2::ZERO;

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

            if let Ok(data) = state.reciever.as_ref().unwrap().try_recv() {
                joystick_input_axes = data;
            }

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
