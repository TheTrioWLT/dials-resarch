#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod ball;
mod dial;
mod frame;
mod window;

pub mod audio;
pub mod config;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// The name of the ball to use for this experient
    pub active_ball: Option<String>,

    /// Where the output data gets stored to once the experiment is done
    pub output_data_path: Option<String>,

    pub balls: Vec<Ball>,
    pub dials: Vec<Dial>,
    pub alarms: Vec<Alarm>,
}

#[derive(Serialize, Deserialize)]
pub struct Ball {
    pub name: String,
    /// The action that this ball takes.
    ///
    /// Can be one of `random_direction`, `random_speed`, or `random_direction_change`
    pub action: BallAction,

    /// How quickly a ball configured for `random_direction_change` changes speed.
    pub random_direction_change_time: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct Dial {
    /// The name of the alarm this dial uses
    pub alarm: String,

    pub start: f32,
    pub end: f32,
}

#[derive(Serialize, Deserialize)]
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

pub enum BallAction {
    /// The ball moves at a random speed
    RandomSpeed,

    /// The ball moves in a random direction from the crosshair
    RandomDirection,

    /// The ball changes directions randomly in a certain interval
    RandomDirectionChange,
}

impl Serialize for BallAction {
    fn serialize<S>(&self, se: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            BallAction::RandomSpeed => "random_direction",
            BallAction::RandomDirection => "random_speed",
            BallAction::RandomDirectionChange => "random_direction_change",
        };
        s.serialize(se)
    }
}

impl<'de> Deserialize<'de> for BallAction {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
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
}
const DEFAULT_INPUT_PATH: &str = "./config.toml";

pub fn run() {
    let mut config = match std::fs::read_to_string(DEFAULT_INPUT_PATH) {
        Ok(toml) => match toml::from_str(&toml) {
            Ok(t) => t,
            Err(e) => {
                panic!();
            }
        },
        /*for i in 0..num_dials {
            let dial = Dial::new(
                i + 1,
                50.0,
                DialRange::new(i as f32 * 200.0, i as f32 * 200.0 + range_size),
                (i + 1).to_string().chars().next().unwrap(),
            );
            dials.push(dial);
        }*/
        Err(_) => {
            let range_size = 4000.0;
            let config = Config {
                balls: vec![Ball {
                    name: "Ball 1".to_owned(),
                    action: BallAction::RandomSpeed,
                    random_direction_change_time: Some(4.0),
                }],
                dials: (1u32..=5)
                    .map(|i| Dial {
                        start: i as f32 * 200.0,
                        end: i as f32 * 200.0 + range_size,
                        alarm: i.to_string(),
                    })
                    .collect(),

                alarms: (1u32..=5)
                    .map(|i| Alarm {
                        name: i.to_string(),
                        audio_path: "alarm.wav".to_owned(),
                        clear_key: char::from_digit(i, 10).unwrap(),
                    })
                    .collect(),

                active_ball: Some("Ball 1".to_owned()),
                output_data_path: None,
            };
            // Write out default config if none existed before
            let toml = toml::to_string(&config).unwrap();
            std::fs::write(DEFAULT_INPUT_PATH, &toml).unwrap();

            config
        }
    };
    validate_config(&mut config);

    window::draw_gui(&config);
}

fn validate_config(config: &mut Config) {
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
            println!("available alarms are {alarm_name:?}");
            std::process::exit(1);
        }
    }
    for alarm in &mut config.alarms {
        alarm.clear_key = alarm
            .clear_key
            .to_uppercase()
            .to_string()
            .chars()
            .next()
            .unwrap();
    }
}
