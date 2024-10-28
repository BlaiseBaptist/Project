use iced::widget::pane_grid::Configuration;
use iced::widget::{button, canvas, column, container, pane_grid, slider, text, Container};
use iced::{mouse, Fill, Rectangle, Renderer, Theme};
#[derive(Debug, Clone)]
enum Pane {
    Graph,
    Text(String),
    Slider,
    Controls,
}
#[derive(Debug, Clone, Copy)]
enum Message {
    Resize(pane_grid::ResizeEvent),
    Move(pane_grid::DragEvent),
    XShift(f32),
    YShift(f32),
    Save,
}
struct App {
    panes: pane_grid::State<Pane>,
	graph: Graph,
}
impl Default for App {
    fn default() -> App {
        App::new()
    }
}
impl App {
    fn new() -> Self {
        let config = Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.5,
            a: Box::new(Configuration::Split {
                axis: pane_grid::Axis::Horizontal,
                ratio: 0.5,
                a: Box::new(Configuration::Pane(Pane::Graph)),
                b: Box::new(Configuration::Pane(Pane::Controls)),
            }),
            b: Box::new(Configuration::Split {
                axis: pane_grid::Axis::Horizontal,
                ratio: 0.5,
                a: Box::new(Configuration::Pane(Pane::Slider)),
                b: Box::new(Configuration::Pane(Pane::Text(
                    "moving panes with the things that will be done on them pretty cool I think"
                        .to_string(),
                ))),
            }),
        };
        let g_state = pane_grid::State::with_configuration(config);
        App {
            panes: g_state,
           	graph: Graph::new(function(1000),0.0,0.0),
        }
    }
    fn view(&self) -> Container<Message> {
        let grid = pane_grid(&self.panes, |_pane, state, _minimized| {
            let title_bar = pane_grid::TitleBar::new(container(text(" Title"))).style(style::title);
            pane_grid::Content::<Message>::new(match state {
                Pane::Graph => container(
                    canvas(self.graph.clone())
                        .width(Fill)
                        .height(Fill),
                )
                .padding(10)
                .style(style::graph),
                Pane::Text(t) => container(text(t))
                    .style(style::text)
                    .padding(10)
                    .width(Fill)
                    .height(Fill),
                Pane::Slider => container(column!(
                    slider(0.0..=100.0, self.graph.x_shift, Message::XShift),
                    slider(0.0..=100.0, self.graph.y_shift, Message::YShift)
                ))
                .style(style::graph)
                .padding(10)
                .width(Fill)
                .height(Fill),
                Pane::Controls => container(column!(
                    button(text("save")).on_press(Message::Save),
                ))
                .style(style::text)
                .padding(10)
                .width(Fill)
                .height(Fill),
            })
            .title_bar(title_bar)
        })
        .spacing(10)
        .on_resize(10, Message::Resize)
        .on_drag(Message::Move);
        container(grid).style(style::app_s).padding(10)
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Resize(e) => self.panes.resize(e.split, e.ratio),
            Message::Move(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.drop(pane, target)
            }
            Message::Move(_) => {}
            Message::XShift(s) => self.graph.x_shift = s,
            Message::YShift(s) => self.graph.y_shift = s,
            Message::Save => println!("{:?}",self.graph.values)
        }
    }
}
#[derive(Clone)]
struct Graph {
    values: Vec<f32>,
    x_scale: f32,
    y_scale: f32,
    x_shift: f32,
    y_shift: f32,
}
impl Graph {
    fn new(values: Vec<f32>, x_shift: f32, y_shift: f32) -> Graph {
        //probably make the values positive or enforce that
        Graph {
            values,
            x_scale: 0.2,
            y_scale: 20.0,
            x_shift,
            y_shift,
        }
    }
}
impl<Message> canvas::Program<Message> for Graph {
    type State = Vec<f32>;
    fn draw(
        &self,
        _state: &Self::State,
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
                style: canvas::Style::Solid(style::graph(&THEME).text_color.unwrap()),
            },
        );
        vec![frame.into_geometry()]
    }
    fn update(
        &self,
        _state: &mut Self::State,
        _event: canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
       (canvas::event::Status::Ignored, None) 
    }
}
#[allow(dead_code)]
mod style {
    use iced::widget::container;
    use iced::{Border, Theme};
    pub fn text(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.primary.base.text),
            background: Some(palette.primary.base.color.into()),
            border: Border {
                width: 2.0,
                color: palette.secondary.base.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
    pub fn title(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.primary.strong.color.into()),
            border: Border {
                width: 1.0,
                color: palette.secondary.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
    pub fn graph(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 2.0,
                color: palette.primary.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
    pub fn app_s(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 1.0,
                color: palette.background.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
}
fn function(x_size: usize) -> Vec<f32> {
    (0..x_size)
        .map(|x| ((x as f32 * 0.01).sin() + 1.0))
        .collect()
}
const THEME: Theme = Theme::Dark;
fn main() {
    let _ = iced::application("Graph", App::update, App::view).run();
}
