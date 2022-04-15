use egui::{epaint::CircleShape, Color32, Pos2, Stroke};
use glium::glutin;

use crate::frame::Frame;
use crate::projectile::Projectile;
use crate::projectile::ProjectileDrawData;

//Uses the glium and glutin to make window.
fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glium example");

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

    let mut dial_angle: f32 = 0.0;

    let mut projectile = Projectile::default(); // Need to find better way to initialize this.
    let mut started = false; //If reading has started

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let mut quit = false;

            let needs_repaint = egui_glium.run(&display, |egui_ctx| {
                //Making a window inside the window, it cannot go away from the parent window
                //
                //Gotta figure out why the circle i make in this window, stays in the main window
                //:)
                //egui::Window::new("Window").show(egui_ctx, |ui| {
                //    let rect = egui::Rect {
                //        min: egui::Pos2 { x: 30.0, y: 30.0 },
                //        max: egui::Pos2 { x: 80.0, y: 80.0 },
                //    };
                //    let pos_circle = egui::Pos2::new(80.0, 40.0);
                //    let circle = egui::epaint::CircleShape::filled(
                //        pos_circle,
                //        20.0,
                //        egui::Color32::DARK_RED,
                //    );

                //    ui.painter().add(egui::Shape::Circle(circle));
                //});

                //Main area

                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    let frame = tracking_frame(egui_ctx, ui, &mut projectile, &mut started);
                    crosshair(&frame, ui);
                    let painter = ui.painter();

                    let window_rect = egui_ctx.available_rect();

                    let sadlfjaso = window_rect.left_bottom();

                    let dial_y_offset_percent = 0.03;
                    let dial_y_offset = dial_y_offset_percent * window_rect.height();
                    let dial_height_percent = 0.3;
                    let dial_total_max_width_percent = 0.6;
                    let num_dials = 20;
                    let dial_max_radius = (window_rect.width() * dial_total_max_width_percent)
                        / (num_dials as f32 * 2.0);

                    let dial_width_percent = 1.0 / (num_dials as f32 + 1.0);

                    let mut dial_radius = dial_height_percent * window_rect.height() / 2.0;

                    if dial_radius > dial_max_radius {
                        dial_radius = dial_max_radius;
                    }

                    for dial in 1..=num_dials {
                        let dial_pos_x = dial as f32 * dial_width_percent * window_rect.width();
                        let dial_center = sadlfjaso
                            + Pos2::new(dial_pos_x, -dial_radius - dial_y_offset).to_vec2();

                        painter.add(egui::Shape::Circle(CircleShape::stroke(
                            dial_center,
                            dial_radius,
                            Stroke::new(2.0, Color32::LIGHT_GREEN),
                        )));

                        let ticks = 16;
                        let tick_radius = 2.0;
                        let tick_inset = tick_radius * 2.0;
                        let tick_inset_radius = dial_radius - tick_inset;
                        let dist = std::f32::consts::TAU / ticks as f32;

                        for i in 0..ticks {
                            let angle = i as f32 * dist;
                            let x = tick_inset_radius * f32::cos(angle);
                            let y = tick_inset_radius * f32::sin(angle);
                            let position = Pos2::new(x + dial_center.x, y + dial_center.y);

                            painter.add(egui::Shape::Circle(CircleShape::filled(
                                position,
                                tick_radius,
                                Color32::LIGHT_YELLOW,
                            )));
                        }

                        let dial_angle_radians = dial_angle.to_radians();
                        let end_position = Pos2::new(
                            dial_center.x + tick_inset_radius * f32::cos(dial_angle_radians),
                            dial_center.y + tick_inset_radius * f32::sin(dial_angle_radians),
                        );

                        painter.add(egui::Shape::LineSegment {
                            points: [dial_center, end_position],
                            stroke: Stroke::new(2.0, Color32::WHITE),
                        });
                    }

                    dial_angle += 1.0;

                    if dial_angle >= 360.0 {
                        dial_angle = dial_angle - 360.0;
                    }
                });

                //egui::SidePanel::left("left").show(egui_ctx, |ui| {
                //    ui.label("Left Program");
                //});
                //egui::SidePanel::right("right").show(egui_ctx, |ui| {
                //    ui.label("Right Program");
                //});
                //egui::CentralPanel::default().show(egui_ctx, |ui| {
                //    ui.label("Main Program");
                //});
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
                use glium::Surface as _;
                let mut target = display.draw();

                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                target.clear_color(color[0], color[1], color[2], color[3]);

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
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                egui_glium.on_event(&event);

                display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            _ => {
                // Not efficient :)
                display.gl_window().window().request_redraw();
            }
        }
        display.gl_window().window().request_redraw();
    });
}

//Handles the movement of the ball
fn tracking_frame(
    egui_ctx: &egui::Context,
    ui: &mut egui::Ui,
    projectile: &mut Projectile,
    started: &mut bool,
) -> egui::Rect {
    ui.label("Yolo");

    let painter = ui.painter();

    let frame = draw_frame(egui_ctx, painter);

    let draw_data = ProjectileDrawData {
        frame,
        width_pos: frame.center(),
        height_pos: frame.center_top(),
    };
    if ui.input().key_down(egui::Key::Space) {
        //When to start the program
        *started = true;
    }
    if !*started {
        projectile.centered(&painter, &draw_data);
    } else {
        projectile.draw(&painter, &draw_data);
    }
    frame
}

//This function should be in its own file
//
//It may be better to have another structure hold the frame made + crosshair in the center of such
fn draw_frame(egui_ctx: &egui::Context, painter: &egui::Painter) -> egui::Rect {
    let window_rect = egui_ctx.available_rect();

    let rec_top_left = egui::Pos2::new(window_rect.width() * 0.20, 0.0);
    let rec_bottom_right = egui::Pos2::new(window_rect.width() * 0.80, window_rect.height() * 0.70);

    let frame = egui::Rect::from_min_max(rec_top_left, rec_bottom_right);

    let stroke = egui::epaint::Stroke::new(1.0, egui::Color32::WHITE);

    let rect = egui::epaint::RectShape::stroke(frame, 0.0, stroke);

    painter.add(egui::Shape::Rect(rect));

    frame
}

fn crosshair(frame: &egui::Rect, ui: &mut egui::Ui) {
    let painter = ui.painter();
    let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

    let v_top_pos = egui::Pos2::new(
        frame.center().x,
        (frame.center().y - frame.height() * 0.05).abs(),
    );
    let v_bot_pos = egui::Pos2::new(frame.center().x, frame.center().y + frame.height() * 0.05);

    painter.add(egui::Shape::LineSegment {
        points: [v_top_pos, v_bot_pos],
        stroke,
    });

    let h_left_pos = egui::Pos2::new(
        (frame.center().x - frame.width() * 0.05).abs(),
        frame.center().y,
    );

    let h_right_pos = egui::Pos2::new(frame.center().x + frame.width() * 0.05, frame.center().y);

    painter.add(egui::Shape::LineSegment {
        points: [h_left_pos, h_right_pos],
        stroke,
    });
}
