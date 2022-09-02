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

#[derive(Serialize, Deserialize, Debug)]
pub struct Ball {
    pub name: String,
    /// The action that this ball takes.
    ///
    /// Can be one of `random_direction`, `random_speed`, or `random_direction_change`
    pub action: BallAction,

    /// How quickly a ball configured for `random_direction_change` changes speed.
    pub random_direction_change_time: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dial {
    /// The name of the alarm this dial uses
    pub alarm: String,

    /// The start of the "in-range"
    pub start: f32,
    /// The end of the "in-range"
    pub end: f32,
    /// The rate at which the dial drifts
    pub rate: f32,
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

#[derive(Copy, Clone, Debug)]
pub enum BallAction {
    /// The ball moves at a random speed
    RandomSpeed,

    /// The ball moves in a random direction from the crosshair
    RandomDirection,

    /// The ball changes directions randomly in a certain interval
    RandomDirectionChange,
}

impl Default for Config {
    fn default() -> Self {
        let range_size = 4000.0;
        Config {
            balls: vec![Ball {
                name: "Ball 1".to_owned(),
                action: BallAction::RandomSpeed,
                random_direction_change_time: Some(4.0),
            }],
            dials: (1u32..=5)
                .map(|i| Dial {
                    alarm: i.to_string(),
                    start: i as f32 * 200.0,
                    end: i as f32 * 200.0 + range_size,
                    rate: 50.0,
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
        }
    }
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
