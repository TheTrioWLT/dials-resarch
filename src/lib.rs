use anyhow::{bail, Result};
use audio::AudioManager;
use dial::{Dial, DialRange};
use eframe::epaint::Vec2;
use lazy_static::lazy_static;
use output::SessionOutput;
use std::{
    collections::HashMap,
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};
use tracking_widget::BoxColor;

use app::{AppState, DialsApp};

use crate::{ball::Ball, output::AlarmReaction};
use gilrs::{Event, Gilrs};

mod app;
mod audio;
mod ball;
mod config;
mod dial;
mod dial_widget;
mod dialog_popup;
mod output;
mod tracking_widget;

/// The default path to the program configuration file
pub const DEFAULT_CONFIG_PATH: &str = "./config.toml";
/// The default path for the program's trial output CSV
pub const DEFAULT_OUTPUT_PATH: &str = "./trial.csv";

lazy_static! {
    /// The global state of the application
    static ref STATE: Mutex<AppState> = Mutex::new(AppState::default());
}

/// Creates a new [`eframe`] window, and spawns worker threads to run the dials research application
///
/// This can fail if the configuration file is invalid, audio files cannot be loaded, or audio playback issues.
pub fn run() -> Result<()> {
    // Parse or generate the configuration file
    let mut config = if let Ok(toml) = std::fs::read_to_string(DEFAULT_CONFIG_PATH) {
        match toml::from_str(&toml) {
            Ok(t) => t,
            Err(e) => {
                dialog_popup::show(
                    "Configuration Error",
                    "Failed to parse configuration file",
                    format!("{e}"),
                )
                .unwrap();

                bail!("Failed to parse configuration file: {}", e);
            }
        }
    } else {
        // Write out default config if none existed before
        let config = config::Config::default();
        let toml = toml::to_string(&config)?;
        std::fs::write(DEFAULT_CONFIG_PATH, toml)?;

        config
    };

    validate_config(&mut config)?;

    let audio = AudioManager::new()?;

    // Maps alarm names to alarm structs
    let alarms: HashMap<&str, &config::Alarm> =
        config.alarms.iter().map(|d| (d.name.as_str(), d)).collect();

    // Loads the audio for each alarm
    for alarm in alarms.values() {
        if let Err(e) = audio.preload_file(&alarm.audio_path) {
            let message = format!("Failed to load {}\n{e}", &alarm.audio_path);

            dialog_popup::show("Audio Load Error", "Failed to load audio file", message).unwrap();

            bail!(
                "Failed to load audio file `{}`: {e}\nDoes the file exist?",
                &alarm.audio_path
            );
        }
    }

    // Generates a Vec<Vec<Dial>> that represents rows of dials, from the configuration
    let dial_rows: Vec<_> = (0..)
        // Loop through each row
        .zip(config.dial_rows.iter())
        .map(|(row_id, row)| {
            (0..)
                // Loop through each dial in the row
                .zip(row.dials.iter())
                .map(|(col_id, dial)| {
                    let alarm = alarms[dial.alarm.as_str()];
                    Dial::new(
                        row_id,
                        col_id,
                        DialRange::new(dial.start, dial.end),
                        alarm,
                        dial.alarm_time,
                    )
                })
                .collect()
        })
        .collect();

    {
        let mut state = STATE.lock().unwrap();

        if let AppState::Running(state) = &mut *state {
            // Assign all of the values that we have created from the configuration file
            // because these had to come with defaults since it is static
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
        } else {
            panic!("App always in the running state on startup");
        }
    }

    // Our "model" runs in a separate thread and shares state
    thread::spawn(move || model(&STATE, audio));

    let options = eframe::NativeOptions {
        transparent: true,
        vsync: true,
        maximized: true,
        initial_window_size: Some(Vec2::new(1920.0 / 2.0, 1080.0 / 2.0)),
        ..eframe::NativeOptions::default()
    };

    // Actually creates the eframe window for our application
    eframe::run_native(
        "Dials App",
        options,
        Box::new(move |cc| Box::new(DialsApp::new(cc, &STATE))),
    )
    .unwrap();

    Ok(())
}

/// Our program's actual internal model, as opposed to the "view" which is our UI
fn model(state: &Mutex<AppState>, audio: AudioManager) {
    // Make instance of the crate that takes care of the joystick inputs.
    let mut gilrs = Gilrs::new().unwrap();

    let mut last_update = Instant::now();

    // The time after the last dial alarm was acknowledged until
    // the "Trial Complete!" splash screen is shown.
    const SPLASH_SCREEN_DELAY: Duration = Duration::from_secs(10);
    // This is set to true when the state.num_dials_done is equal to the number of dials
    let mut is_done = false;
    // Represents the instant where the last dial alarm was acknowledged. The initial
    // value here is thrown away, and re-set when the state.num_alarms_done is max.
    let mut last_dial_time = Instant::now();

    // Outputs the type of device that is detected by Gilrs.
    // If information is not recognized by library then it will output the default OS provided name
    for (_id, gamepad) in gilrs.gamepads() {
        log::info!(
            "Joystick {} detected: {:?}",
            gamepad.name(),
            gamepad.power_info()
        );
    }

    // Keep track of the joystick's axes
    // This is only called if joystick is the input mode otherwise goes to keyboard state input
    let mut joystick_input_axes = Vec2::default();

    let total_num_alarms = {
        let mut state = state.lock().unwrap();
        if let AppState::Running(state) = &mut *state {
            state.dial_rows.iter().map(|r| r.len()).sum()
        } else {
            panic!();
        }
    };

    // This allows us to request state transitions inside of the loop
    let mut new_appstate = None;

    loop {
        thread::sleep(Duration::from_millis(2));

        let delta_time = last_update.elapsed().as_secs_f32();

        let mut state = state.lock().unwrap();

        match &mut *state {
            AppState::Running(state) => {
                // We need an extra vec here so that we can mutably borrow both `state.dial_rows` and
                // `state.queued_alarms` at the same time
                let mut alarms = Vec::new();

                // Update all dials
                for row in state.dial_rows.iter_mut() {
                    for dial in row.iter_mut() {
                        if let Some(alarm) = dial.update(delta_time, &audio) {
                            alarms.push(alarm);
                        }
                    }
                }

                state.queued_alarms.extend(alarms);

                // Takes the event detected by the joystick being used.
                // Events detected can be 3 types of axes:
                //   -X
                //   -Y
                //   -Z
                // The only ones we care about are X and Y.
                // We then take the amount the joystick moves. This is already filtered by on a scale
                // -1 to 1 where 0 is centered or not moving.
                while let Some(Event { event, .. }) = gilrs.next_event() {
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

                // Depending on the type of input specified in the config file it will then proceed to
                // decide to either use the joystick axes or the keyboard.
                // If needed more will be added, such as Mouse input.
                let input_axes = match state.input_mode {
                    config::InputMode::Joystick => joystick_input_axes,
                    config::InputMode::Keyboard => state.input_axes,
                };

                state.ball.update(input_axes, delta_time);

                // Process key presses/alarm reactions
                if let Some(key) = state.pressed_key {
                    if let Some(alarm) = state.queued_alarms.pop_front() {
                        let millis = alarm.time.elapsed().as_millis() as u32;

                        let current_rms_error = state.ball.current_rms_error();
                        let dial =
                            &mut state.dial_rows[alarm.row_id as usize][alarm.col_id as usize];

                        let reaction = AlarmReaction::new(
                            dial.alarm_name().clone(),
                            millis,
                            alarm.correct_key == key,
                            key,
                            current_rms_error,
                        );

                        dial.reset();
                        audio.stop(alarm.id);

                        state.session_output.add_reaction(reaction);

                        state.num_alarms_done += 1;

                        //Tell the state that a key was pressed after an alarm went off.
                        state.tracking_state.blink(Some(BoxColor::Green));
                    }

                    if !is_done && state.num_alarms_done == total_num_alarms {
                        state.session_output.write_to_file();

                        log::info!(
                            "wrote session output to file: {}",
                            state.session_output.output_path
                        );

                        last_dial_time = Instant::now();
                        is_done = true;
                    }

                    state.pressed_key = None;
                }

                //If key detected then start running time
                state.tracking_state.update(delta_time);
                // We have a delay before going to the end screen
                if is_done && last_dial_time.elapsed() >= SPLASH_SCREEN_DELAY {
                    // Change the state to Done and therefore show the splash screen
                    new_appstate = Some(AppState::Done);
                }
            }
            AppState::Done => {}
        }

        // If we have requested a state change, perform it
        if let Some(new_state) = new_appstate.take() {
            *state = new_state;
        }

        last_update = Instant::now();
    }
}

/// Validates a config file, or exits the program with an error printed to the command line on how
/// to fix the validation
fn validate_config(config: &mut config::Config) -> Result<()> {
    let alarm_names: Vec<_> = config.alarms.iter().map(|b| &b.name).collect();
    // Loops through each dial and checks if its corresponding alarm exists in the map
    for row in &config.dial_rows {
        for dial in &row.dials {
            let alarm_name = &dial.alarm;
            if !alarm_names.contains(&alarm_name) {
                let message = format!(
                    "Alarm `{alarm_name}` is missing!\nAvailable alarms are: {alarm_names:?}"
                );

                dialog_popup::show(
                    "Configuration Error",
                    "Invalid configuration",
                    message.clone(),
                )
                .unwrap();

                bail!(message);
            }
        }
    }

    Ok(())
}
