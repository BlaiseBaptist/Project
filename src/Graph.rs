pub mod Graph {
    use iced::{mouse, widget::canvas, Rectangle, Renderer, Theme};
	use crate::Style;
    #[derive(Clone)]
    pub struct FloatingGraph {
        pub values: Vec<f32>,
        pub x_scale: f32,
        pub y_scale: f32,
        pub x_shift: f32,
        pub y_shift: f32,
    }
    impl FloatingGraph {
        pub fn new(values: Vec<f32>, x_shift: f32, y_shift: f32) -> FloatingGraph {
            FloatingGraph {
                values,
                x_scale: 0.2,
                y_scale: 20.0,
                x_shift,
                y_shift,
            }
        }
    }
    impl<Message> canvas::Program<Message> for FloatingGraph {
        type State = Vec<f32>;
        fn draw(
            &self,
            _state: &Self::State,
            renderer: &Renderer,
            _theme: &Theme,
            bounds: Rectangle,
            _cursor: mouse::Cursor,
        ) -> Vec<canvas::Geometry> {
            let mut frame = canvas::Frame::new(renderer, bounds.size());
            let scale = canvas::path::lyon_path::geom::euclid::Transform2D::new(
                self.x_scale,
                0.0,
                0.0,
                self.y_scale,
                self.x_shift,
                self.y_shift,
            );
            let mut lines = canvas::path::Builder::new();
            lines.move_to(iced::Point::new(0.0, self.values[0]));
            for i in 1..self.values.len() {
                lines.line_to(iced::Point::new(i as f32, self.values[i]));
            }
            frame.stroke(
                &lines.build().transform(&scale),
                canvas::Stroke {
                    line_cap: canvas::LineCap::Round,
                    line_dash: canvas::LineDash {
                        offset: 0,
                        segments: &[1.0, 0.0],
                    },
                    line_join: canvas::LineJoin::Bevel,
                    width: 1.0,
                    style: canvas::Style::Solid(Style::Style::graph(&Style::Style::THEME).text_color.unwrap()),
                },
            );
            vec![frame.into_geometry()]
        }
    }
}
