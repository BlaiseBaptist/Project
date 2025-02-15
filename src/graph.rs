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
        pub values: Vec<f32>,
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
            let height = -scale.m32 / scale.m22;
            let mut lines = canvas::path::Builder::new();
            self.values
                .iter()
                .skip(start)
                .take(end)
                .enumerate()
                .for_each(|(i, value)| {
                    lines.line_to(Point::new(
                        i as f32,
                        if value < &height { *value } else { height },
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
            let text_size = 16.0;
            if end < 10 {
                return vec![frame.into_geometry()];
            }
            if height < 5.0 {
                return vec![frame.into_geometry()];
            }
            println!(
                "y_shift: {}, y_shift2: {}, scaled_height: {}, real_height: {}",
                state.y_shift,
                scale.m32,
                height,
                bounds.size().height,
            );
            const y_lines: u32 = 3;
            for y in 1..y_lines + 1 {
                let line_height = y as f32 * (bounds.size().height) / (1 + y_lines) as f32;
                canvas::Text {
                    color: theme.palette().text,
                    content: (height * (1.0 - (y as f32 / (y_lines + 1) as f32))).to_string(),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Bottom,
                    line_height: 1.0.into(),
                    position: Point::new(0.0, line_height),
                    size: text_size.into(),
                    shaping: iced::widget::text::Shaping::Basic,
                }
                .draw_with(|path, color| frame.stroke(&path, stroke.with_color(color)));
                let graph_line = canvas::Path::line(
                    Point::new(bounds.size().width, line_height),
                    Point::new(0.0, line_height),
                );
                frame.stroke(&graph_line, stroke.with_color(theme.palette().text));
            }
            for x in (start..=end)
                .step_by((end - start) / 10)
                .enumerate()
                .skip(1)
            {
                canvas::Text {
                    color: theme.palette().text,
                    content: (x.1).to_string(),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: match x.0 {
                        0 => iced::alignment::Horizontal::Left,
                        10 => iced::alignment::Horizontal::Right,
                        _ => iced::alignment::Horizontal::Center,
                    },
                    vertical_alignment: iced::alignment::Vertical::Bottom,
                    line_height: 1.0.into(),
                    position: Point::new(
                        x.0 as f32 * bounds.size().width / 10.0,
                        bounds.size().height - 10.0,
                    ),
                    size: text_size.into(),
                    shaping: iced::widget::text::Shaping::Basic,
                }
                .draw_with(|path, color| frame.stroke(&path, stroke.with_color(color)));
                let graph_line = canvas::Path::line(
                    Point::new(
                        x.0 as f32 * bounds.size().width / 10.0,
                        bounds.size().height - 20.0,
                    ),
                    Point::new(x.0 as f32 * bounds.size().width / 10.0, 0.0),
                );
                frame.stroke(&graph_line, stroke.with_color(theme.palette().text));
            }
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
            if !cursor.is_over(bounds) {
                return (event_status, None);
            }
            if let Event::Mouse(e) = event {
                match e {
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
                    mouse::Event::CursorMoved { position } => {
                        if let Some(last_position) = state.last_mouse_click {
                            state.x_shift = position.x - last_position.x;
                            state.y_shift = position.y - last_position.y;
                        }
                    }
                    _ => {}
                }
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
                x_scale: 2.0,
                y_scale: 600.0,
                x_shift: 0.0,
                y_shift: -10.0,
                last_mouse_click: None,
            }
        }
    }
}
