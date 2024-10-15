use iced::widget::{canvas, pane_grid, PaneGrid};
use iced::{mouse, Color, Rectangle, Renderer, Theme};

struct App {
    panes: pane_grid::State<Pane>,
}
#[derive(Debug, Clone, Copy)]
enum Pane {
    Graph,
}
#[derive(Debug, Clone, Copy)]
enum Message {
    Resize(pane_grid::ResizeEvent),
}

impl App {
    fn view(&self) -> PaneGrid<Message> {
        pane_grid(&self.panes, |_pane, state, _minimized| {
            pane_grid::Content::<Message>::new(match state {
                Pane::Graph => canvas(Graph::new(function(100))),
            })
        }).on_resize(10, Message::Resize)
        .into()
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Resize(re) => println!("{:?}",re.ratio),
        }
    }
}
impl Default for App {
    fn default() -> App {
        App {
            panes: pane_grid::State::new(Pane::Graph).0,
        }
    }
}
struct Graph {
    values: Vec<f32>,
    x_scale: f32,
    y_scale: f32,
	x_shift: f32,
	y_shift: f32,
}
impl Graph {
    fn new(values: Vec<f32>) -> Graph {
        Graph {
            values: values,
            x_scale: 1.0,
            y_scale: 10.0,
			x_shift: 10.0,
			y_shift: 10.0,
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
            self.x_shift,
            self.y_shift,
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
    let _ = iced::run("A cool counter", App::update, App::view);
}
