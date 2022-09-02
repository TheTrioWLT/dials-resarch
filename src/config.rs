use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Where the output data gets stored to once the experiment is done
    pub output_data_path: Option<String>,

    pub ball: Ball,
    pub dials: Vec<Dial>,
    pub alarms: Vec<Alarm>,
}

#[derive(Serialize, Deserialize)]
pub struct Ball {
    pub random_direction_change_time_min: f32,
    pub random_direction_change_time_max: f32,
}

#[derive(Serialize, Deserialize)]
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

impl Default for Config {
    fn default() -> Self {
        let range_size = 4000.0;
        println!("Happ");
        Config {
            ball: Ball {
                random_direction_change_time_min: 1.0,
                random_direction_change_time_max: 8.0,
            },
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

            output_data_path: None,
        }
    }
}
