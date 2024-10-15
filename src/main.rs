#[derive(Default)]
struct Counter {
    value: usize,
}
#[derive(Debug, Clone, Copy)]
enum Message {
    ValueChange(isize),
}

use iced::widget::canvas::path::lyon_path::geom::euclid;
use iced::widget::{button, canvas, column, row, text, Column};
use iced::{mouse, Color, Rectangle, Renderer, Theme};
impl Counter {
    fn view(&self) -> Column<Message> {
        column![canvas(Graph::new(function(100)))]
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::ValueChange(v) => self.value = self.value.overflowing_add_signed(v).0,
        }
    }
}

struct Graph {
    values: Vec<f32>,
    x_scale: f32,
    y_scale: f32,
}
impl Graph {
    fn new(values: Vec<f32>) -> Graph {
        Graph {
            values: values,
            x_scale: 1.0,
            y_scale: 10.0,
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
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let scale = canvas::path::lyon_path::geom::euclid::Transform2D::new(
            self.x_scale,
            0.0,
            0.0,
            self.y_scale,
            0.0,
            0.0,
        );
        let mut lines = canvas::path::Builder::new();
        lines.move_to(iced::Point::new(0.0, self.values[0]));
        for i in 1..self.values.len() {
            lines.line_to(iced::Point::new(i as f32, self.values[i]));
        }
        frame.stroke(
            &lines.build().transform(&scale),
            canvas::Stroke {
                line_cap: canvas::LineCap::Round,
                line_dash: canvas::LineDash {
                    offset: 0,
                    segments: &[1.0, 0.0],
                },
                line_join: canvas::LineJoin::Bevel,
                width: 5.0,
                style: canvas::Style::Solid(Color::WHITE),
            },
        );
        vec![frame.into_geometry()]
    }
}
fn function(x_size: usize) -> Vec<f32> {
    (0..x_size)
        .map(|x| ((x as f32 * std::f32::consts::PI * 0.05).sin() + 1.0))
        .collect()
}
fn main() {
    let _ = iced::run("A cool counter", Counter::update, Counter::view);
}
