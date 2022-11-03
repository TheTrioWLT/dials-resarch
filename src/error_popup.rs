use eframe::egui::Vec2;
use eframe::{egui, NativeOptions};

pub fn show(title: impl AsRef<str>, heading: impl Into<String>, message: impl Into<String>) -> ! {
    let native_options = NativeOptions {
        always_on_top: true,
        resizable: false,
        initial_window_size: Some(Vec2::new(400.0, 160.0)),
        ..NativeOptions::default()
    };

    let heading = heading.into();
    let message = message.into();
    eframe::run_native(
        title.as_ref(),
        native_options,
        Box::new(move |cc| Box::new(ErrorPopupWindow::new(heading, message, cc))),
    );
}

#[derive(Default)]
struct ErrorPopupWindow {
    heading: String,
    message: String,
}

impl ErrorPopupWindow {
    fn new(heading: String, message: String, cc: &eframe::CreationContext<'_>) -> Self {
        Self::style(cc);

        Self { heading, message }
    }

    fn style(cc: &eframe::CreationContext<'_>) {
        let mut style = egui::style::Style::default();
        style.visuals = egui::style::Visuals::dark();
        cc.egui_ctx.set_style(style);
    }
}

impl eframe::App for ErrorPopupWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                ui.heading(&self.heading);
                ui.add_space(20.0);

                ui.label(&self.message);
                ui.add_space(20.0);

                if ui.button("Ok").clicked() {
                    frame.quit();
                }
            });
        });
    }
}
