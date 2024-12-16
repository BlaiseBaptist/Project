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
                values: vec![],
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
            let len = self.values.len();
            if len < bounds.size().width as usize {
                return vec![];
            }
            let mut frame = canvas::Frame::new(renderer, bounds.size());
            let scale = canvas::path::lyon_path::geom::euclid::Transform2D::new(
                1.0 / state.x_scale,
                0.0,
                0.0,
                -bounds.size().height / state.y_scale,
                state.x_shift,
                bounds.size().height + state.y_shift,
            );
            let end: usize = ((bounds.size().width - state.x_shift) * state.x_scale) as usize;
            let start: usize = (-state.x_shift * state.x_scale) as usize;
            let height = (-scale.m32 / scale.m22) as u32;
            let mut lines = canvas::path::Builder::new();
            self.values
                .iter()
                .skip(start)
                .take(end)
                .enumerate()
                .for_each(|(i, value)| {
                    lines.line_to(Point::new(
                        i as f32,
                        if value < &height {
                            *value as f32
                        } else {
                            height as f32 * 1.1
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
            let axis_numbers: Vec<String> = (start as usize..=end as usize)
                .step_by((end - start) / 1)
                .map(|v| v.to_string())
                .collect();
            let horizontal_axis_text = canvas::Text {
                color: theme.palette().text,
                content: axis_numbers.join(" to "),
                font: iced::Font::DEFAULT,
                horizontal_alignment: iced::alignment::Horizontal::Left,
                vertical_alignment: iced::alignment::Vertical::Bottom,
                line_height: 1.0.into(),
                position: Point::new(0.0, bounds.size().height - 10.0),
                size: 20.0.into(),
                shaping: iced::widget::text::Shaping::Basic,
            };
            frame.stroke(&lines.build().transform(&scale), stroke);
            horizontal_axis_text
                .draw_with(|path, color| frame.stroke(&path, stroke.with_color(color)));
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
            if self.values.len() < bounds.size().width as usize {
                return (event_status, None);
            }
            match event {
                Event::Mouse(e) => match e {
                    mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Pixels { x, y },
                    } => {
                        state.x_scale -= x / 10.0;
                        state.y_scale *= 1.0 + (y / 1000.0);
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
            if state.y_scale < 2.0 {
                state.y_scale = 2.0;
            }
            if state.x_scale > self.values.len() as f32 * 10.0 / (bounds.size().width) {
                state.x_scale = self.values.len() as f32 * 10.0 / (bounds.size().width);
            }
            if state.x_scale < 0.5 {
                state.x_scale = 0.5;
            }
            return (event_status, None);
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
                x_scale: 2.0,
                y_scale: 600.0,
                x_shift: 0.0,
                y_shift: -10.0,
                last_mouse_click: None,
            }
        }
    }
}
