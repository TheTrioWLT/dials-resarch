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

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let mut quit = false;

            let needs_repaint = egui_glium.run(&display, |egui_ctx| {
                //Making a window inside the window, it cannot go away from the parent window
                //
                //Gotta figure out why the circle i make in this window, stays in the main window
                //:)
                egui::Window::new("Window").show(egui_ctx, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(), |shape| {
                        shape.label("Hello");
                        let pos_circle = egui::Pos2::new(40.0, 40.0);
                        let circle = egui::epaint::CircleShape::filled(
                            pos_circle,
                            20.0,
                            egui::Color32::DARK_RED,
                        );

                        shape.painter().add(egui::Shape::Circle(circle));
                    });
                });

                //Main area
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.label("Yolo");
                    ui.horizontal(|ui| {
                        ui.button("This is an empty button").clicked();
                        ui.button("Another Button, Whaaaaat").clicked();
                    });
                    ui.vertical(|ui| {
                        ui.button("vertical Button Babyyyy").clicked();
                        ui.add_space(33.0);
                        ui.set_width(50.0);
                        ui.button("FAT Vertical Button 2, BABYYYY").clicked();
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
