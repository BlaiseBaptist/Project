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
        pub values: Vec<[u8; 4]>,
        pub port: Box<dyn port::port::Port>,
        pub converter: converter,
    }
    impl Graph {
        pub fn new(port: Box<dyn port::port::Port>) -> Graph {
            Graph {
                values: vec![],
                port,
                converter: converter::be_f32,
            }
        }
        pub fn swap_endianness(&mut self) {
            self.converter = self.converter.swap();
        }
        pub fn push(&mut self, v: [u8; 4]) {
            self.values.push(v);
        }
        pub fn get_values(&self) -> Vec<f32> {
            self.values
                .iter()
                .map(|x| self.converter.convert(*x))
                .collect()
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
            let start = (-scale.m31 / scale.m11) as usize;
            let end = ((bounds.size().width - scale.m31) / scale.m11) as usize + 1;
            let step_size = match bounds.size().width / (scale.m11 * 10000.0) {
                v if v <= 1.0 => 1.0,
                v => v,
            };
            let height = -scale.m32 / scale.m22;
            let bottom = (bounds.size().height - 20.0 - scale.m32) / scale.m22;
            let mut lines = canvas::path::Builder::new();
            self.values
                .iter()
                .enumerate()
                .skip(start)
                .take(end)
                .step_by(step_size as usize)
                .for_each(|(i, value)| {
                    lines.line_to(Point::new(
                        i as f32,
                        match self.converter.convert(*value) {
                            v if v > height => height,
                            v if v < bottom => bottom,
                            v => v,
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
            let background_stroke: canvas::Stroke = canvas::Stroke {
                line_cap: canvas::LineCap::Butt,
                line_dash: canvas::LineDash {
                    offset: 0,
                    segments: &[7.0, 3.0],
                },
                line_join: canvas::LineJoin::Miter,
                width: 0.5,
                style: canvas::Style::Solid(theme.palette().background),
            };
            frame.stroke(&lines.build().transform(&scale), stroke);
            let text_size = 16.0;
            let y_lines = 10;
            let x_lines = 10;
            for y in 0..y_lines {
                let value_sep = 10_f32.powi((height - bottom).log10() as i32);
                let line_value = value_sep * ((bottom / value_sep).floor() + y as f32);
                let line_pos = (scale.m22 * line_value) + scale.m32;
                canvas::Text {
                    color: theme.palette().primary,
                    content: format!("{0:.1e}", line_value),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Bottom,
                    line_height: 1.0.into(),
                    position: Point::new(1.0, line_pos),
                    size: text_size.into(),
                    shaping: iced::widget::text::Shaping::Basic,
                }
                .draw_with(|path, color| frame.stroke(&path, stroke.with_color(color)));
                let graph_line = canvas::Path::line(
                    Point::new(bounds.size().width, line_pos),
                    Point::new(0.0, line_pos),
                );
                frame.stroke(&graph_line, background_stroke);
            }
            for x in -1..x_lines {
                let value_sep = 10_f32.powi(((end - start) as f32).log10().floor() as i32);
                let line_value = value_sep * x as f32;
                let line_pos = (scale.m11 * line_value) + scale.m31;
                if x == 0 {
                    let _line_debug = format!(
                        "sep: {:.0e}, value: {}, pos: {}, start: {}",
                        value_sep, line_value, line_pos, start
                    );
                    println!("{}", _line_debug);
                }
                canvas::Text {
                    color: theme.palette().primary,
                    content: format!("{0:.1e}", line_value),
                    font: iced::Font::DEFAULT,
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Bottom,
                    line_height: 1.0.into(),
                    position: Point::new(line_pos, bounds.size().height - 10.0),
                    size: text_size.into(),
                    shaping: iced::widget::text::Shaping::Basic,
                }
                .draw_with(|path, color| frame.stroke(&path, stroke.with_color(color)));
                let graph_line = canvas::Path::line(
                    Point::new(line_pos, bounds.size().height - 20.0),
                    Point::new(line_pos, 0.0),
                );
                frame.stroke(&graph_line, background_stroke);
            }
            let _debug = format!(
                "start: {}, end: {}, step: {},  scale: {}, shift: {}, min: {}, max: {}",
                start, end, step_size, scale.m11, scale.m31, bottom, height
            );
            // println!("{}", _debug);
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
            if !cursor.is_over(bounds) {
                return (event_status, None);
            }
            if let Event::Mouse(e) = event {
                match e {
                    mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Pixels { x, y },
                    } => {
                        state.x_scale += x / 400.0;
                        state.y_scale -= y / 400.0;
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
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq)]
    pub enum converter {
        be_f32,
        le_f32,
        be_u32,
        le_u32,
        u8_to_string,
    }
    impl converter {
        fn convert(&self, data: [u8; 4]) -> f32 {
            match self {
                converter::be_f32 => f32::from_be_bytes(data),
                converter::le_f32 => f32::from_le_bytes(data),
                converter::be_u32 => u32::from_be_bytes(data) as f32,
                converter::le_u32 => u32::from_le_bytes(data) as f32,
                converter::u8_to_string => data
                    .into_iter()
                    .map(|b| char::from(b))
                    .collect::<String>()
                    .parse::<f32>()
                    .unwrap_or(0.0),
            }
        }
        fn swap(&self) -> Self {
            match self {
                converter::be_f32 => converter::le_f32,
                converter::le_f32 => converter::be_u32,
                converter::be_u32 => converter::le_u32,
                converter::le_u32 => converter::u8_to_string,
                converter::u8_to_string => converter::be_f32,
            }
        }
        pub fn name(&self) -> String {
            match self {
                converter::be_f32 => "be_f32",
                converter::le_f32 => "le_f32",
                converter::be_u32 => "be_u32",
                converter::le_u32 => "le_u32",
                converter::u8_to_string => "u8_to_string",
            }
            .to_string()
        }
    }
}
