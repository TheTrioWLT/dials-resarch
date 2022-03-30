use std::ops::Add;

use glium::glutin;

//Uses the glium and glutin to make window.
fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("Research Program");

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
                    ui.centered_and_justified(|ui| {
                        let mut dials_list: Vec<egui::epaint::CircleShape> = Vec::new();
                        let dial_number = 3;

                        for i in 0..dial_number {
                            let posx = ui.min_rect().min.x + (-100.0 + 100.0 * i as f32);
                            let posy = ui.min_rect().min.y + 200.0;

                            let circe_pos = egui::pos2(posx, posy);

                            let circle = egui::epaint::CircleShape::filled(
                                circe_pos,
                                50.0,
                                egui::Color32::BLUE,
                            );

                            dials_list.push(circle);
                        }

                        for circle in dials_list.iter() {
                            ui.painter().add(egui::Shape::Circle(*circle));
                        }
                    });
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

            _ => (),
        }
    });
}
