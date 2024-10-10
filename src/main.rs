#[derive(Default)]
struct Counter {
    value: i64,
}
#[derive(Debug, Clone, Copy)]
enum Message {
    ValueChange(f64),
    NewValue(f64),
}

use iced::widget::{button, canvas, column, row, slider, text, Column};
use iced::{mouse, Color, Rectangle, Renderer, Theme};
impl Counter {
    fn view(&self) -> Column<Message> {
        column![
            /*text(self.value).size(100),
            row![
                button("+").on_press(Message::ValueChange(1.0)),
                button("*").on_press(Message::ValueChange(self.value as f64)),
            ],
            row![
                button("-").on_press(Message::ValueChange(-1.0)),
                button("/").on_press(Message::ValueChange((self.value as f64) * -0.5)),
            ],
            slider(
                0.0..=(self.value as f64) * 1.5 + 10.0,
                self.value as f64,
                Message::NewValue
       		),*/
			canvas(Graph::new((1..100).map(|x| x as f32).collect())),
        ]
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::ValueChange(v) => self.value += v.round() as i64,
            Message::NewValue(v) => self.value = v.round() as i64,
        }
    }
}

struct Graph {
    x_size: usize,
    y_size: usize,
    values: Vec<f32>,
	scale: f32,
}
impl Graph {
    fn new(values: Vec<f32>) -> Graph {
        Graph {
            x_size: values.len()*10,
            y_size: values.clone().into_iter().reduce(f32::max).unwrap() as usize * 10,
            values: values,
			scale: 10.0,
        }
    }
}
impl<Message> canvas::Program<Message> for Graph {
    type State = ();
    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(
            renderer,
            iced::Size::new(self.x_size as f32, self.y_size as f32),
        );
        let style = canvas::Stroke {
            line_cap: canvas::LineCap::Butt,
            line_dash: canvas::LineDash {
                offset: 0,
                segments: &[1.0,0.0],
            },
            line_join: canvas::LineJoin::Round,
            width: 10.0,
            style: canvas::Style::Solid(Color::BLACK),
        };
        for i in 1..self.values.len() {
            frame.stroke(
                &canvas::Path::line(
                    iced::Point::new((i - 1) as f32 * self.scale, self.values[i - 1] * self.scale),
                    iced::Point::new((i) as f32 * self.scale, self.values[i] * self.scale),
                ),
                style,
            );
    	}    
        vec![frame.into_geometry()]
    }
}
fn main() {
    let _ = iced::run("A cool counter", Counter::update, Counter::view);
}
