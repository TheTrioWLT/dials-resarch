#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod dial;
mod window;

pub mod audio;
pub mod config;

use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct Config {
    pub dials: Vec<Dial>,
    pub balls: Vec<Ball>,
    pub alarms: Vec<Alarm>,

    /// The name of the ball to use for this experient
    pub active_ball: Option<String>,

    /// Where the output data gets stored to once the experiment is done
    #[serde(default = "out_path_default")]
    pub output_data_path: String,
}

fn out_path_default() -> String {
    "data.csv".into()
}

#[derive(Deserialize)]
pub struct Ball {
    pub name: String,
    /// The action that this ball takes.
    ///
    /// Can be one of `random_direction`, `random_speed`, or `random_direction_change`
    #[serde(deserialize_with = "de_action")]
    pub action: BallAction,

    /// How quickly a ball configured for `random_direction_change` changes speed.
    pub random_direction_change_time: Option<f64>,
}

#[derive(Deserialize)]
pub struct Dial {
    pub in_range_zone: f64,
    pub out_of_range_zone: f64,
    pub unit_step: f64,
    pub alarm: String,
}

#[derive(Deserialize)]
pub struct Alarm {
    /// The user defined name of this alarm. Used to match up which alarm is being used in
    /// [`Dial::alarm`]
    pub name: String,

    /// The path to the audio file for this alarm
    pub audio_path: String,

    /// The key that clears this alarm.
    /// Case insensitive single letter or standard key code like `<Escape>`
    pub clear_key: String,
}

pub enum BallAction {
    /// The ball moves at a random speed
    RandomSpeed,

    /// The ball moves in a random direction from the crosshair
    RandomDirection,

    /// The ball changes directions randomly in a certain interval
    RandomDirectionChange,
}

fn de_action<'de, D>(de: D) -> Result<BallAction, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(de)?;
    match s.as_str() {
        "random_direction" => Ok(BallAction::RandomDirection),
        "random_speed" => Ok(BallAction::RandomSpeed),
        "random_direction_change" => Ok(BallAction::RandomDirectionChange),
        _ => Err(serde::de::Error::custom(format!(
            "Unknown ball action `{s}`"
        ))),
    }
}

pub fn run() {
    window::draw_gui();
}
