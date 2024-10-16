use iced::widget::pane_grid::Configuration;
use iced::widget::{canvas, container, pane_grid, Container};
use iced::{mouse, Color, Fill, Rectangle, Renderer, Theme};
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
    Move(pane_grid::DragEvent),
}

impl App {
    fn new() -> Self {
        let config = Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.5,
            a: Box::new(Configuration::Pane(Pane::Graph)),
            b: Box::new(Configuration::Pane(Pane::Graph)),
        };
        let g_state = pane_grid::State::with_configuration(config);
        App { panes: g_state }
    }
    fn view(&self) -> Container<Message> {
        let grid = pane_grid(&self.panes, |_pane, state, _minimized| {
            pane_grid::Content::<Message>::new(match state {
                Pane::Graph => container(canvas(Graph::new(function(1000))).width(Fill).height(Fill))
                    .padding(10)
                    .style(|_| { style::pane_focused }(&Theme::Dracula)),
            })
        })
        .on_resize(10, Message::Resize)
        .on_drag(Message::Move);
        container(grid)
            .style(|_| { style::title_bar_active }(&Theme::Dracula))
            .into()
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Resize(e) => self.panes.resize(e.split, e.ratio),
            Message::Move(_e) => todo!(),
        }
    }
}
impl Default for App {
    fn default() -> App {
        App::new()
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
            x_scale: 2.0,
            y_scale: 20.0,
            x_shift: 0.0,
            y_shift: 0.0,
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
                width: 1.0,
                style: canvas::Style::Solid(Color::WHITE),
            },
        );
        vec![frame.into_geometry()]
    }
}
#[allow(dead_code)]
mod style {
    use iced::widget::container;
    use iced::{Border, Theme};

    pub fn title_bar_active(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        container::Style {
            text_color: Some(palette.background.strong.text),
            background: Some(palette.background.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        container::Style {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.primary.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn pane_active(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 2.0,
                color: palette.background.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();

        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 2.0,
                color: palette.primary.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
}
fn function(x_size: usize) -> Vec<f32> {
    (0..x_size)
        .map(|x| ((x  as f32 * std::f32::consts::PI * 0.05).sin() + 1.0))
        .collect()
}
fn main() {
    let _ = iced::run("Graph", App::update, App::view);
}
