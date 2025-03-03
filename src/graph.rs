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
            let mut frame = canvas::Frame::new(renderer, bounds.size());
            let scale = canvas::path::lyon_path::geom::euclid::Transform2D::new(
                10f32.powf(state.x_scale),
                0.0,
                0.0,
                -bounds.size().height * 10f32.powf(state.y_scale),
                state.x_shift,
                bounds.size().height + state.y_shift,
            );
            let start: usize = (-scale.m31 / scale.m11) as usize;
            let end: usize = ((bounds.size().width - scale.m31) / scale.m11) as usize + 1;
            let step_size = if end - start > self.values.len() {
                (scale.m11 / 2.0) as usize + 1
            } else {
                1
            };
            let height = -scale.m32 / scale.m22;
            let bottom = (bounds.size().height - 20.0 - scale.m32) / scale.m22;
            let mut lines = canvas::path::Builder::new();
            self.values
                .iter()
                .skip(start)
                .take(end)
                .enumerate()
                .step_by(step_size)
                .for_each(|(i, value)| {
                    lines.line_to(Point::new(
                        i as f32,
                        match value {
                            v if v > &height => height,
                            v if v < &bottom => bottom,
                            _ => *value,
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
            let text_size = 16.0;
            let y_lines = (bounds.size().height / 100.0) as i32;
            let y_sep = bounds.size().height / (y_lines as f32 + 1.0);
            let x_lines = (bounds.size().width / 100.0) as i32;
            let x_sep = bounds.size().width / (x_lines as f32 + 1.0);
            for y in 1..y_lines + 1 {
                let line_sep = bounds.size().height - y_sep * y as f32;
                canvas::Text {
                    color: theme.palette().text,
                    content: format!("{0:.2e}", (line_sep - scale.m32) / scale.m22),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Bottom,
                    line_height: 1.0.into(),
                    position: Point::new(0.0, line_sep),
                    size: text_size.into(),
                    shaping: iced::widget::text::Shaping::Basic,
                }
                .draw_with(|path, color| frame.stroke(&path, stroke.with_color(color)));
                let graph_line = canvas::Path::line(
                    Point::new(bounds.size().width, line_sep),
                    Point::new(0.0, line_sep),
                );
                frame.stroke(&graph_line, stroke.with_color(theme.palette().text));
            }
            for x in 1..x_lines + 1 {
                let line_sep = bounds.size().width - x_sep * x as f32;
                canvas::Text {
                    color: theme.palette().text,
                    content: format!("{0:.2e}", (line_sep - scale.m31) / scale.m11),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Bottom,
                    line_height: 1.0.into(),
                    position: Point::new(line_sep, bounds.size().height - 10.0),
                    size: text_size.into(),
                    shaping: iced::widget::text::Shaping::Basic,
                }
                .draw_with(|path, color| frame.stroke(&path, stroke.with_color(color)));
                let graph_line = canvas::Path::line(
                    Point::new(line_sep, bounds.size().height - 20.0),
                    Point::new(line_sep, 0.0),
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
                        state.x_scale += x / 1000.0;
                        state.y_scale -= y / 1000.0;
                        state.x_shift *= 10_f32.powf(x / 1000.0);
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
                x_scale: -1.0,
                y_scale: -1.0,
                x_shift: 0.0,
                y_shift: 0.0,
                last_mouse_click: None,
            }
        }
    }
}
