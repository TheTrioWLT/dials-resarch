use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use egui::{epaint::RectShape, Color32};
use glium::glutin;
use glutin::event::{ElementState, VirtualKeyCode};

use crate::{
    dial::{
        Dial, DialDrawData, DialRange, DialReaction, DIALS_MAX_WIDTH_PERCENT, DIAL_HEIGHT_PERCENT,
        DIAL_Y_OFFSET_PERCENT,
    },
    frame::Frame,
};

const WINDOW_COLOR: Color32 = Color32::from_rgb(27, 27, 27);

//Uses the glium and glutin to make window.
fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("Dials Research Program");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}

/// Map a key press `k` to to its dial number, or None if `k` is not a dial
macro_rules! key_to_dial_num {
    ($k:expr, $($case:path, $lit:literal),+) => {
        match $k {
            $($case => Some($lit),)+
            _ => None,
        }
    };
}

//Draws the gui, window, images, labels etc...
pub fn draw_gui(config: &crate::Config) {
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    //Initiates the display area
    let mut egui_glium = egui_glium::EguiGlium::new(&display);

    // Maps alarm names to alarm structs
    let alarms: HashMap<&str, &crate::Alarm> =
        config.alarms.iter().map(|d| (d.name.as_str(), d)).collect();

    let mut dials: Vec<_> = config
        .dials
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let alarm = alarms[d.alarm.as_str()];
            Dial::new(i, 50.0, DialRange::new(d.start, d.end), alarm.clear_key)
        })
        .collect();

    let mut last_frame = Instant::now();

    let mut frame = Frame::new();

    // These are being used sort of like input axes, but these are Pos2(positive, negative) in that
    // in the case of arrow keys, you can press down both the up and down arrows at the same time.
    // So if that were true, the input_y would be Pos2(1.0, 1.0) and we would do math on both axes
    // combined.
    let mut input_y = egui::Pos2::new(0.0, 0.0);
    let mut input_x = egui::Pos2::new(0.0, 0.0);

    let mut pressed_key: Option<char> = None;

    // This stores the current dial alarms and resets that the user needs to perform
    let mut queued_alarms = VecDeque::new();

    event_loop.run(move |event, _, control_flow| {
        // This is the corrected version of the input_y and input_x
        let input_axes = egui::Pos2::new(input_x.y - input_x.x, input_y.x - input_y.y);

        let mut redraw = || {
            let quit = false;

            let needs_repaint = egui_glium.run(&display, |egui_ctx| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    // Make this work continously on Windows
                    egui_ctx.request_repaint();

                    let painter = ui.painter();
                    let now = Instant::now();
                    let delta_time = now.duration_since(last_frame).as_secs_f32();
                    last_frame = now;

                    let window_rect = egui_ctx.available_rect();
                    let window_height = window_rect.height();
                    let window_width = window_rect.width();

                    // ----------- Draw the window background -----------
                    painter.add(egui::Shape::Rect(RectShape::filled(
                        window_rect,
                        egui::Rounding::none(),
                        WINDOW_COLOR,
                    )));

                    let window_left_bottom = window_rect.left_bottom();

                    // ----------------- Draw the dials -----------------
                    let dial_y_offset = DIAL_Y_OFFSET_PERCENT * window_height;
                    let dial_max_radius =
                        (window_width * DIALS_MAX_WIDTH_PERCENT) / (dials.len() as f32 * 2.0);

                    let dial_width_percent = 1.0 / (dials.len() as f32 + 1.0);

                    let mut dial_radius = DIAL_HEIGHT_PERCENT * window_height / 2.0;

                    if dial_radius > dial_max_radius {
                        dial_radius = dial_max_radius;
                    }

                    let dial_draw_data = DialDrawData {
                        y_offset: dial_y_offset,
                        radius: dial_radius,
                        dial_width_percent,
                        window_width,
                        window_left_bottom,
                    };

                    for dial in dials.iter_mut() {
                        if let Some(alarm) = dial.update(delta_time) {
                            queued_alarms.push_back(alarm);
                        }

                        dial.draw(painter, &dial_draw_data);
                    }

                    if let Some(key) = pressed_key {
                        if let Some(alarm) = queued_alarms.pop_front() {
                            let millis = alarm.time.elapsed().as_millis() as u32;

                            let reaction = DialReaction::new(
                                alarm.dial_id,
                                millis,
                                alarm.correct_key == key,
                                key,
                            );

                            dials[alarm.dial_id].reset();

                            println!("{reaction:?}");
                        }
                    }

                    // ----------------- Draw the frame -----------------
                    frame.update(&input_axes, delta_time);
                    frame.draw(painter, &window_rect, delta_time);

                    // Reset the pressed key since it was released last time
                    pressed_key = None;
                });
            });

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                let mut target = display.draw();

                // draw things behind egui here

                egui_glium.paint(&display, &mut target);

                // draw things on top of egui here

                target.finish().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;
                match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            glutin::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    } => {
                        // Highly inefficient but good enough for testing
                        if state == ElementState::Pressed {
                            match keycode {
                                VirtualKeyCode::Up => {
                                    input_y.x = 1.0;
                                }
                                VirtualKeyCode::Down => {
                                    input_y.y = 1.0;
                                }
                                VirtualKeyCode::Left => {
                                    input_x.x = 1.0;
                                }
                                VirtualKeyCode::Right => {
                                    input_x.y = 1.0;
                                }
                                _ => {}
                            }
                        } else {
                            match keycode {
                                VirtualKeyCode::Up => {
                                    input_y.x = 0.0;
                                }
                                VirtualKeyCode::Down => {
                                    input_y.y = 0.0;
                                }
                                VirtualKeyCode::Left => {
                                    input_x.x = 0.0;
                                }
                                VirtualKeyCode::Right => {
                                    input_x.y = 0.0;
                                }
                                k => {
                                    use VirtualKeyCode::*;

                                    let maybe_dial = key_to_dial_num!(
                                        k, Key1, '1', Key2, '2', Key3, '3', Key4, '4', Key5, '5',
                                        Key6, '6', Key7, '7', Key8, '8', Key9, '9', A, 'A', B, 'B',
                                        C, 'C', D, 'D', E, 'E', F, 'F', G, 'G', H, 'H', I, 'I', J,
                                        'J', K, 'K', L, 'L', M, 'M', N, 'N', O, 'O', P, 'P', Q,
                                        'Q', R, 'R', S, 'S', T, 'T', U, 'U', V, 'V', W, 'W', X,
                                        'X', Y, 'Y', Z, 'Z'
                                    );

                                    if let Some(dial_num) = maybe_dial {
                                        pressed_key = Some(dial_num);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                egui_glium.on_event(&event);

                display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            _ => {}
        }
    });
}
