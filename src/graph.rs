pub mod graph {
    use crate::port;
    use iced::{mouse, widget::canvas, Rectangle, Renderer, Theme};
    #[derive(Debug)]
    pub struct Graph {
        pub values: Vec<u32>,
        pub x_scale: f32,
        pub y_scale: f32,
        pub x_shift: f32,
        pub y_shift: f32,
        pub port: Box<dyn port::port::Port>,
    }
    impl Graph {
        pub fn new(x_shift: f32, y_shift: f32, port: Box<dyn port::port::Port>) -> Graph {
            Graph {
                values: vec![0],
                x_scale: 1.0,
                y_scale: 1.0,
                x_shift,
                y_shift,
                port,
            }
        }
        pub fn swap_endianness(&mut self) {
            self.port.swap_endianness()
        }
    }
    impl<Message> canvas::Program<Message> for Graph {
        type State = Vec<u32>;
        fn draw(
            &self,
            _state: &Self::State,
            renderer: &Renderer,
            theme: &Theme,
            bounds: Rectangle,
            _cursor: mouse::Cursor,
        ) -> Vec<canvas::Geometry> {
            let mut frame = canvas::Frame::new(renderer, bounds.size());
            let len = self.values.len();
            let start = match len.checked_sub(bounds.size().width as usize) {
                Some(v) => v,
                _ => 0,
            };
            let scale = canvas::path::lyon_path::geom::euclid::Transform2D::new(
                self.x_scale,
                0.0,
                0.0,
                -self.y_scale * bounds.size().height / 2u32.pow(31u32) as f32,
                self.x_shift,
                self.y_shift + bounds.size().height,
            );
            let mut lines = canvas::path::Builder::new();
            let graph_values = &self.values[start..];
            lines.move_to((0.0, graph_values[0] as f32).into());
            graph_values
                .iter()
                .enumerate()
                .skip(1)
                .for_each(|(i, value)| lines.line_to(iced::Point::new(i as f32, *value as f32)));
            let stroke: canvas::Stroke = canvas::Stroke {
                line_cap: canvas::LineCap::Butt,
                line_dash: canvas::LineDash {
                    offset: 0,
                    segments: &[1.0, 0.0],
                },
                line_join: canvas::LineJoin::Miter,
                width: 1.0,
                style: canvas::Style::Solid(theme.palette().text),
            };
            frame.stroke(&lines.build().transform(&scale), stroke);
            vec![frame.into_geometry()]
        }
    }
}
