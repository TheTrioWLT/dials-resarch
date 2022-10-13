use eframe::egui;
use eframe::egui::Vec2;

pub struct ErrorPopup {
    title: String,
    heading: String,
    message: String,
}

impl ErrorPopup {
    pub fn new(
        title: impl Into<String>,
        heading: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            heading: heading.into(),
            message: message.into(),
        }
    }

    pub fn show(self) {
        let mut native_options = eframe::NativeOptions::default();
        native_options.always_on_top = true;
        native_options.initial_window_size = Some(Vec2::new(400.0, 160.0));
        native_options.resizable = false;
        let title = self.title.clone();
        eframe::run_native(
            &title,
            native_options,
            Box::new(move |cc| Box::new(ErrorPopupWindow::new(self, cc))),
        );
    }
}

#[derive(Default)]
struct ErrorPopupWindow {
    heading: String,
    message: String,
}

impl ErrorPopupWindow {
    fn new(error_data: ErrorPopup, cc: &eframe::CreationContext<'_>) -> Self {
        Self::style(cc);

        Self {
            heading: error_data.heading,
            message: error_data.message,
        }
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
