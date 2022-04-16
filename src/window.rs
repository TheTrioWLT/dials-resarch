use crate::frame::Frame;
use crate::projectile::Projectile;
use glium::glutin;

use crate::dial::{
    Dial, DialDrawData, DIALS_MAX_WIDTH_PERCENT, DIAL_HEIGHT_PERCENT, DIAL_Y_OFFSET_PERCENT,
};

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
        dials.push(Dial::new(i + 1));
    }

    let mut projectile = Projectile::default(); // Need to find better way to initialize this.

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let quit = false;

            let needs_repaint = egui_glium.run(&display, |egui_ctx| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    let painter = ui.painter();
                    let mut new_frame = Frame::new(egui_ctx);

                    new_frame.draw_frame(painter);
                    projectile.draw(painter, &new_frame.window_rect);

                    //let frame = tracking_frame(egui_ctx, ui, &mut projectile, &mut started);
                    //crosshair(&frame, ui);

                    let window_rect = egui_ctx.available_rect();
                    let window_height = window_rect.height();
                    let window_width = window_rect.width();

                    let window_left_bottom = window_rect.left_bottom();

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
                    };

                    for dial in dials.iter_mut() {
                        dial.draw(painter, &dial_draw_data);

                        dial.increment_value(10);
                    }
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
            _ => {}
        }

        // Not efficient :)
        display.gl_window().window().request_redraw();
    });
}
