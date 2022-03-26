pub mod audio;
pub mod config;

pub struct Config {
    pub dials: Vec<Dial>,
    pub balls: Vec<Ball>,
    pub active_ball: Option<String>,
}

pub struct Ball {
    pub name: String,
    /// The action that this ball takes.
    ///
    /// Can be one of `random_direction`, `random_speed`, or `random_direction_change`
    pub action: String,

    /// How quickly a ball configured for `random_direction_change` changes speed.
    pub random_direction_change_time: Option<f64>,
}

pub struct Dial {
    pub in_range_zone: f64,
    pub out_of_range_zone: f64,
    pub alarm_file: String,
    pub unit_step: f64,
}
