const FRAME_HEIGHT_PRCNT: f32 = 0.70;
const FRAME_WIDTH_PRCNT: f32 = 0.20;

const FRAME_BORDER_WIDTH: f32 = 1.0;
const FRAME_BORDER_COLOR: egui::Color32 = egui::Color32::WHITE;

const FRAME_EDGE_ROUND: f32 = 0.0;

const V_CROSSHAIR_OFFSET: f32 = 0.05;
const H_CROSSHAIR_OFFSET: f32 = 0.05;

///Holds the new frame made inside a window for main program where projectile moves.

pub struct Frame {
    pub window_rect: egui::Rect,
    pub crosshair: Vec<egui::Shape>,
}

impl Frame {
    ///Only the Context of the window will be used in order to construct what is needed.
    pub fn new(egui_ctx: &egui::Context) -> Self {
        let window_rect = egui_ctx.available_rect();

        let rec_top_left =
            egui::Pos2::new(window_rect.width() * FRAME_WIDTH_PRCNT, window_rect.top());
        let rec_bottom_right = egui::Pos2::new(
            window_rect.width() * FRAME_WIDTH_PRCNT * 4.0,
            window_rect.height() * FRAME_HEIGHT_PRCNT,
        );

        let frame = egui::Rect::from_min_max(rec_top_left, rec_bottom_right);

        //let rect = egui::epaint::RectShape::stroke(frame, 0.0, stroke);

        let stroke = egui::epaint::Stroke::new(FRAME_BORDER_WIDTH, egui::Color32::WHITE);

        let mut crosshair = Vec::new();

        //Making the lines for the crosshair
        //
        //Needs fixing, this way of constructing is not ideal
        let v_top_pos = egui::Pos2::new(
            frame.center().x,
            (frame.center().y - frame.height() * V_CROSSHAIR_OFFSET).abs(),
        );
        let v_bot_pos = egui::Pos2::new(
            frame.center().x,
            frame.center().y + frame.height() * V_CROSSHAIR_OFFSET,
        );

        crosshair.push(egui::Shape::LineSegment {
            points: [v_top_pos, v_bot_pos],
            stroke,
        });

        let h_left_pos = egui::Pos2::new(
            (frame.center().x - frame.width() * H_CROSSHAIR_OFFSET).abs(),
            frame.center().y,
        );

        let h_right_pos = egui::Pos2::new(
            frame.center().x + frame.width() * H_CROSSHAIR_OFFSET,
            frame.center().y,
        );

        crosshair.push(egui::Shape::LineSegment {
            points: [h_left_pos, h_right_pos],
            stroke,
        });

        Frame {
            window_rect: frame,
            crosshair,
        }
    }

    ///Draws the frame assuming the user setup the Frame structure properly.
    pub fn draw_frame(&mut self, painter: &egui::Painter) {
        let stroke = egui::epaint::Stroke::new(FRAME_BORDER_WIDTH, FRAME_BORDER_COLOR);

        let rect = egui::epaint::RectShape::stroke(self.window_rect, FRAME_EDGE_ROUND, stroke);

        painter.add(egui::Shape::Rect(rect));

        for rect in self.crosshair.iter() {
            painter.add(rect.to_owned());
        }
    }
}
