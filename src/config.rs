use crate::{ball::BallVelocity, tracking_widget::FeedbackColor};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigTrial {
    /// The key to respond to this experiment trial
    /// A case insensitive character
    pub correct_response_key: char,

    /// Text to display after a correct key was presseed
    pub feedback_text_correct: Option<String>,

    /// Text to display after an incorrect key was pressed
    pub feedback_text_incorrect: Option<String>,

    /// Changes the color of box corresponding to key pressed
    pub feedback_color_correct: Option<FeedbackColor>,

    /// Changes the color of box corresponding to key pressed
    pub feedback_color_incorrect: Option<FeedbackColor>,

    /// The name of the dial which this trial is associated with
    /// [`Dial`]
    pub dial: String,

    /// The name of the alarm which this trial is associated with
    /// [`Alarm`]
    pub alarm: String,

    /// The time at which the dial should drift outside of its range,
    /// and the alarm should sound
    pub alarm_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigDial {
    /// The name (identifier) for this dial, so that it can be referenced by a [`Trial`]
    pub name: String,

    /// The start of the in-range for this dial
    pub range_start: f32,

    /// The end of the in-range for this dial
    pub range_end: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigAlarm {
    /// The user defined name of this alarm. Used to match up which trial it is being used in
    ///
    /// [`Dial::alarm`]
    pub name: String,

    /// The path to the audio file for this alarm
    pub audio_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigBall {
    /// Stores the range for random time.
    ///
    /// This specifically stores the minimum time of the range
    pub random_direction_change_time_min: f32,

    /// Stores the Maximum time for the range.
    pub random_direction_change_time_max: f32,

    /// Specifies the velocity type of the ball;
    ///
    /// -Slow
    /// -Medium
    /// -Fast
    ///
    /// [`BallVelocity`]
    pub ball_velocity: BallVelocity,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Where the output data gets stored to once the experiment is done
    pub output_data_path: Option<String>,

    /// What type of input is desired for the program:
    ///
    /// [`InputMode`]
    pub input_mode: InputMode,

    /// Attributes necessary for the ball that we need
    ///
    /// ['ConfigBall']
    pub ball: ConfigBall,

    /// The trials concerning dials and alarms that the program will execute and respond to
    ///
    /// [`ConfigTrial`]
    pub trials: Vec<ConfigTrial>,

    #[serde(rename = "row")]
    /// Number of rows for dials along with Dial attributes needed.
    ///
    /// ['DialRow'] ['Dial'] for more information
    pub dial_rows: Vec<ConfigDialRow>,

    /// Attributes for the alarms such as what keys stops them and the file to use.
    ///
    /// ['Alarm']
    pub alarms: Vec<ConfigAlarm>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigDialRow {
    /// A row of dials on the GUI
    #[serde(rename = "dial")]
    pub dials: Vec<ConfigDial>,
}

///Default Config file that is made if no "config.json" is detected
impl Default for Config {
    fn default() -> Self {
        let range_size = 4000.0;
        Config {
            ball: ConfigBall {
                random_direction_change_time_min: 1.0,
                random_direction_change_time_max: 8.0,
                ball_velocity: BallVelocity::Slow,
            },
            output_data_path: None,
            input_mode: InputMode::default(),
            trials: (1u32..=6)
                .map(|i| ConfigTrial {
                    correct_response_key: char::from_digit(i, 10).unwrap(),
                    feedback_text_correct: Some(String::from("CORRECT")),
                    feedback_text_incorrect: Some(String::from("INCORRECT")),
                    feedback_color_correct: Some(FeedbackColor::Green),
                    feedback_color_incorrect: Some(FeedbackColor::Red),
                    dial: format!("d{i}"),
                    alarm: format!("a{i}"),
                    alarm_time: 4.0,
                })
                .collect(),
            dial_rows: vec![
                ConfigDialRow {
                    dials: (1u32..=3)
                        .map(|i| ConfigDial {
                            name: format!("d{i}"),
                            range_start: i as f32 * 200.0,
                            range_end: i as f32 * 200.0 + range_size,
                        })
                        .collect(),
                },
                ConfigDialRow {
                    dials: (4u32..=6)
                        .map(|i| ConfigDial {
                            name: format!("d{i}"),
                            range_start: i as f32 * 200.0,
                            range_end: i as f32 * 200.0 + range_size,
                        })
                        .collect(),
                },
            ],
            alarms: (1u32..=6)
                .map(|i| ConfigAlarm {
                    name: format!("a{i}"),
                    audio_path: "alarm.wav".to_owned(),
                })
                .collect(),
        }
    }
}

/// The input mode for controlling the ball
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum InputMode {
    /// Joystick input through [`gilrs`]
    Joystick,
    /// Keyboard input through WASD
    #[default]
    Keyboard,
}
