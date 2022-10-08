use crate::ball::BallVelocity;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Where the output data gets stored to once the experiment is done
    pub output_data_path: Option<String>,

    pub ball: Ball,
    pub dials: Vec<Dial>,
    pub alarms: Vec<Alarm>,
    pub input_mode: InputMode,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ball {
    pub random_direction_change_time_min: f32,
    pub random_direction_change_time_max: f32,
    pub velocity_meter: BallVelocity,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dial {
    /// The name of the alarm this dial uses
    pub alarm: String,

    /// The start of the "in-range"
    pub start: f32,
    /// The end of the "in-range"
    pub end: f32,

    /// The absolute time at which this alarm
    /// should sound, aka. when the dial should drift out of range
    pub alarm_time: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Alarm {
    /// The user defined name of this alarm. Used to match up which alarm is being used in
    /// [`Dial::alarm`]
    pub name: String,

    /// The path to the audio file for this alarm
    pub audio_path: String,

    /// The key that clears this alarm.
    /// Case insensitive single letter
    pub clear_key: char,
}

impl Default for Config {
    fn default() -> Self {
        let range_size = 4000.0;
        Config {
            ball: Ball {
                random_direction_change_time_min: 1.0,
                random_direction_change_time_max: 8.0,
                velocity_meter: BallVelocity::Slow,
            },
            dials: (1u32..=5)
                .map(|i| Dial {
                    alarm: i.to_string(),
                    start: i as f32 * 200.0,
                    end: i as f32 * 200.0 + range_size,
                    alarm_time: 8.0 + (i as f32) * 6.0,
                })
                .collect(),

            alarms: (1u32..=5)
                .map(|i| Alarm {
                    name: i.to_string(),
                    audio_path: "alarm.wav".to_owned(),
                    clear_key: char::from_digit(i, 10).unwrap(),
                })
                .collect(),
            output_data_path: None,
            input_mode: InputMode::default(),
        }
    }
}

impl Serialize for BallVelocity {
    fn serialize<S>(&self, se: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            BallVelocity::Slow => "slow",
            BallVelocity::Medium => "medium",
            BallVelocity::Fast => "fast",
        };
        s.serialize(se)
    }
}

impl<'de> Deserialize<'de> for BallVelocity {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(de)?;
        match s.as_str() {
            "slow" => Ok(BallVelocity::Slow),
            "medium" => Ok(BallVelocity::Medium),
            "fast" => Ok(BallVelocity::Fast),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown ball velocity `{s}`"
            ))),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy)]
pub enum InputMode {
    #[serde(rename = "joystick")]
    Joystick,
    #[default]
    #[serde(rename = "keyboard")]
    Keyboard,
}
