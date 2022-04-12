#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod dial;
mod window;

pub fn run() {
    window::draw_gui();
}
