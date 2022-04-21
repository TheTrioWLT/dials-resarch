#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod ball;
mod dial;
mod frame;
mod window;

pub mod audio;
pub mod config;

const DEFAULT_INPUT_PATH: &str = "./config.toml";

pub fn run() {
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
            let toml = toml::to_string(&config).unwrap();
            std::fs::write(DEFAULT_INPUT_PATH, &toml).unwrap();

            config
        }
    };
    validate_config(&mut config);

    window::draw_gui(&config);
}

fn validate_config(config: &mut config::Config) {
    if let Some(active) = &config.active_ball {
        let ball_names: Vec<_> = config.balls.iter().map(|b| &b.name).collect();
        if !ball_names.contains(&active) {
            println!("active ball `{active}` is missing");
            println!("available balls are {ball_names:?}");
            std::process::exit(1);
        }
    }

    // We need to print `alarm_names` in the event of an error
    #[allow(clippy::needless_collect)]
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
