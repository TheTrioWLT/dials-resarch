#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp_micros()
        .filter(Some("dials-research"), log::LevelFilter::Debug)
        .filter(None, log::LevelFilter::Info)
        .init();
    dials_research::run()
}
