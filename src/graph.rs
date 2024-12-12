pub mod graph {
    use crate::port;
    use iced::{
        mouse,
        widget::canvas,
        widget::canvas::{event, Event},
        Rectangle, Renderer, Theme,
    };
    #[derive(Debug)]
    pub struct Graph {
        pub values: Vec<u32>,
        pub port: Box<dyn port::port::Port>,
    }
    impl Graph {
        pub fn new(port: Box<dyn port::port::Port>) -> Graph {
            Graph {
                values: vec![0],
                port,
            }
        }
        pub fn swap_endianness(&mut self) {
            self.port.swap_endianness()
        }
    }
    impl<Message> canvas::Program<Message> for Graph {
        type State = [f32; 2];
        fn draw(
            &self,
            state: &Self::State,
            renderer: &Renderer,
            theme: &Theme,
            bounds: Rectangle,
            _cursor: mouse::Cursor,
        ) -> Vec<canvas::Geometry> {
            let mut frame = canvas::Frame::new(renderer, bounds.size());
            let len = self.values.len();
            let start = match len.checked_sub((bounds.size().width / state[0].abs()) as usize) {
                Some(v) => v,
                _ => 0,
            };
            let scale = canvas::path::lyon_path::geom::euclid::Transform2D::new(
                state[0].abs(),
                0.0,
                0.0,
                -bounds.size().height / state[1],
                0.0,
                bounds.size().height,
            );
            let height = (-scale.m32 / scale.m22) as u32;
            let mut lines = canvas::path::Builder::new();
            self.values
                .iter()
                .skip(start)
                .enumerate()
                .for_each(|(i, value)| {
                    lines.line_to(iced::Point::new(
                        i as f32,
                        if value < &height {
                            *value as f32
                        } else {
                            height as f32
                        },
                    ))
                });
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
        fn update(
            &self,
            state: &mut Self::State,
            event: Event,
            _bounds: Rectangle,
            _cursor: mouse::Cursor,
        ) -> (event::Status, Option<Message>) {
            match event {
                Event::Mouse(e) => match e {
                    mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Pixels { x, y },
                    } => {
                        state[0] += x / 100.0;
                        state[1] += y;
                    }
                    _ => {}
                },
                _ => {}
            };
            (event::Status::Ignored, None)
        }
    }
}
