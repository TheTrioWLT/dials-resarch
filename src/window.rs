use std::time::Instant;

use egui::{epaint::RectShape, Color32};
use glium::glutin;
use glutin::event::{ElementState, VirtualKeyCode};

use crate::{
    dial::{
        Dial, DialDrawData, DIALS_MAX_WIDTH_PERCENT, DIAL_HEIGHT_PERCENT, DIAL_Y_OFFSET_PERCENT,
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

//Draws the gui, window, images, labels etc...
pub fn draw_gui() {
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    //Initiates the display area
    let mut egui_glium = egui_glium::EguiGlium::new(&display);

    let num_dials = 5;
    let mut dials = Vec::new();

    for i in 0..num_dials {
        let dial = Dial::new(i + 1, (i + 1) as f32 * 100.0);
        dials.push(dial);
    }

    let mut last_frame = Instant::now();

    let mut frame = Frame::new();

    // These are being used sort of like input axes, but these are Pos2(positive, negative) in that
    // in the case of arrow keys, you can press down both the up and down arrows at the same time.
    // So if that were true, the input_y would be Pos2(1.0, 1.0) and we would do math on both axes
    // combined.
    let mut input_y = egui::Pos2::new(0.0, 0.0);
    let mut input_x = egui::Pos2::new(0.0, 0.0);

    event_loop.run(move |event, _, control_flow| {
        // This is the corrected version of the input_y and input_x
        let input_axes = egui::Pos2::new(input_x.y - input_x.x, input_y.x - input_y.y);

        let mut redraw = || {
            let quit = false;

            let needs_repaint = egui_glium.run(&display, |egui_ctx| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
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
                        (window_width * DIALS_MAX_WIDTH_PERCENT) / (num_dials as f32 * 2.0);

                    let dial_width_percent = 1.0 / (num_dials as f32 + 1.0);

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
                        delta_time,
                    };

                    for dial in dials.iter_mut() {
                        dial.draw(painter, &dial_draw_data);
                    }

                    frame.update(&input_axes);
                    frame.draw(painter, &window_rect, delta_time);
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
                                _ => {}
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

        // Not efficient :)
        display.gl_window().window().request_redraw();
    });
}
