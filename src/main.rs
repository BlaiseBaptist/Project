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
            text(self.value).size(100),
            row![
                button("+").on_press(Message::ValueChange(1.0)),
                button("*").on_press(Message::ValueChange(self.value as f64)),
            ],
            row![
                button("-").on_press(Message::ValueChange(-1.0)),
                button("/").on_press(Message::ValueChange((self.value as f64) * -0.5)),
            ],
            slider(-100.0..=100.0, self.value as f64, Message::NewValue),
            canvas(Graph {
                value: self.value as f32,
                x_size: self.value as usize * 2,
                y_size: self.value as usize* 2,
				values: vec![1.0,1.0]
            }),
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
    value: f32,
    x_size: usize,
    y_size: usize,
	values: Vec<f32>
}
impl Graph {
	fn new(values: Vec<f32>) -> Graph{
		Graph{
			value: 0.0, //remove once have a graph
			x_size: values.len(),
			y_size: values.clone().into_iter().reduce(f32::max).unwrap() as usize,
			values: values
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
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, iced::Size::new(self.x_size as f32, self.y_size as f32));
        let graph = canvas::Path::circle(frame.center(), self.value);
        frame.fill(&graph, Color::BLACK);
        vec![frame.into_geometry()]
    }
}

fn main() {
    let _ = iced::run("A cool counter", Counter::update, Counter::view);
}
