#[derive(Default)]
struct Counter {
    value: usize,
}
#[derive(Debug, Clone, Copy)]
enum Message {
    ValueChange(isize),
    NewValue(usize),
}

use iced::widget::{button, canvas, column, row, slider, text, Column};
use iced::{mouse, Color, Rectangle, Renderer, Theme};
impl Counter {
    fn view(&self) -> Column<Message> {
        column![
            text(self.value).size(100),	
            row![
                button("+").on_press(Message::ValueChange(10)),
//                button("*").on_press(Message::ValueChange(self.value as f64)),
            ],
            row![
                button("-").on_press(Message::ValueChange(-10)),
//                button("/").on_press(Message::ValueChange((self.value as f64) * -0.5)),
            ],
			/*
            slider(
                0.0..=(self.value as f64) * 1.5 + 10.0,
                self.value as f64,
                Message::NewValue
       		),*/
			if self.value >= 10{
				canvas(Graph::new(function(self.value )))
			}
			else{
				canvas(Graph::new(function(10)))
			}
        ]
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::ValueChange(v) => self.value = self.value.overflowing_add_signed(v).0,
            Message::NewValue(v) => self.value = v,
        }
    }
}

struct Graph {
    x_size: usize,
    y_size: usize,
    values: Vec<f32>,
	x_scale: f32,
	y_scale: f32,
}
impl Graph {
    fn new(values: Vec<f32>) -> Graph {
        Graph {
            x_size: values.len(),
            y_size: values.clone().into_iter().reduce(f32::max).unwrap() as usize * 40,
            values: values,
			x_scale: 1.0,
			y_scale: 10.0
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
            line_cap: canvas::LineCap::Round,
            line_dash: canvas::LineDash {
                offset: 0,
                segments: &[1.0,0.0],
            },
            line_join: canvas::LineJoin::Bevel,
            width: 5.0,
            style: canvas::Style::Solid(Color::BLACK),
        };
        for i in 1..self.values.len() {
            frame.stroke(
                &canvas::Path::line(
                    iced::Point::new((i - 1) as f32 * self.x_scale, self.values[i - 1] * self.y_scale),
                    iced::Point::new((i) as f32 * self.x_scale, self.values[i] * self.y_scale),
                ),
                style,
            );
    	}    
        vec![frame.into_geometry()]
    }
}
fn function(x_size: usize) -> Vec<f32>{
	(0..x_size).map(|x| ((x as f32 * std::f32::consts::PI * 0.05).sin() + 1.0) ).collect()
}
fn main() {
    let _ = iced::run("A cool counter", Counter::update, Counter::view);
}
