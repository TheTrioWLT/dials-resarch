pub mod audio;
pub mod config;

pub struct Config {
    pub dials: Vec<Dial>,
    pub z_ball_dir_change_min: f64,
    pub z_ball_dir_change_max: f64,
    pub y_ball_speed_min: f64,
    pub y_ball_speed_max: f64,
}

pub struct Dial {
    pub in_range_zone: f64,
    pub out_of_range_zone: f64,
    pub alarm_file: String,
    pub unit_step: f64,
}
