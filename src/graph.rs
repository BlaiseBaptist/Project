pub mod graph {
    use crate::port;
    use iced::{
        mouse,
        widget::canvas,
        widget::canvas::{event, Event},
        Point, Rectangle, Renderer, Theme,
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
        type State = GraphControls;
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
            let start = match len.checked_sub((bounds.size().width * state.x_scale) as usize) {
                Some(v) => v,
                _ => 0,
            };
            let scale = canvas::path::lyon_path::geom::euclid::Transform2D::new(
                1.0 / state.x_scale,
                0.0,
                0.0,
                -bounds.size().height / state.y_scale,
                state.x_shift - 1.0,
                bounds.size().height + state.y_shift - 10.0,
            );
            let height = (-scale.m32 / scale.m22) as u32;
            let mut lines = canvas::path::Builder::new();
            self.values
                .iter()
                .take(start)
                .enumerate()
                .for_each(|(i, value)| {
                    lines.line_to(Point::new(
                        i as f32,
                        if value < &height {
                            *value as f32
                        } else {
                            height as f32 + 10.0
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
            bounds: Rectangle,
            cursor: mouse::Cursor,
        ) -> (event::Status, Option<Message>) {
            let mut event_status = event::Status::Ignored;
            match event {
                Event::Mouse(e) => match e {
                    mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Pixels { x, y },
                    } => {
                        state.x_scale += x / 100.0;
                        state.y_scale += y;
                        event_status = event::Status::Captured;
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        state.last_mouse_click = (|| -> Option<Point> {
                            Some(
                                cursor.position()?
                                    + iced::Vector::new(-state.x_shift, -state.y_shift),
                            )
                        })();
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        state.last_mouse_click = None
                    }
                    mouse::Event::CursorMoved { position } => match state.last_mouse_click {
                        Some(last_position) => {
                            state.x_shift = position.x - last_position.x;
                            state.y_shift = position.y - last_position.y;
                        }
                        None => {}
                    },
                    _ => {}
                },
                _ => {}
            };
            if state.x_scale < 2.0 {
                state.x_scale = 2.0;
            }
            if state.x_scale > self.values.len() as f32 / (bounds.size().width * 2.0) {
                state.x_scale = self.values.len() as f32 / (bounds.size().width * 2.0);
            }
            if state.y_scale < 2.0 {
                state.y_scale = 2.0;
            }
            (event_status, None)
        }
    }
    #[derive(Debug)]
    pub struct GraphControls {
        x_scale: f32,
        y_scale: f32,
        x_shift: f32,
        y_shift: f32,
        last_mouse_click: Option<Point>,
    }
    impl std::default::Default for GraphControls {
        fn default() -> GraphControls {
            GraphControls {
                x_scale: 1.0,
                y_scale: 1.0,
                x_shift: 0.0,
                y_shift: 0.0,
                last_mouse_click: None,
            }
        }
    }
}
