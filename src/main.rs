use iced::{
    widget::{
        button, canvas, column, container, pane_grid, pane_grid::Configuration, pick_list, slider,
        text, text_input, Container,
    },
    Fill,
};
use serialport::SerialPortInfo;
mod graph;
mod style;
#[derive(Debug, Clone)]
enum Pane {
    Graph,
    Text(String),
    Slider,
    Controls,
}
#[derive(Debug, Clone)]
enum Message {
    Resize(pane_grid::ResizeEvent),
    Move(pane_grid::DragEvent),
    XShift(f32),
    YShift(f32),
    Save,
    PathChanged(String),
    ChangePort(String),
    UpdateGraph,
}
struct App {
    panes: pane_grid::State<Pane>,
    graph: graph::graph::FloatingGraph,
    path: String,
    ports: Result<Vec<SerialPortInfo>, serialport::Error>,
    port: Option<String>,
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
            graph: graph::graph::FloatingGraph::new(function(1000), 0.0, 0.0),
            path: "graph1.csv".to_string(),
            ports: serialport::available_ports(),
            port: None::<String>,
        }
    }
    fn view(&self) -> Container<Message> {
        let grid = pane_grid(&self.panes, |_pane, state, _minimized| {
            let title_text: String;
            pane_grid::Content::<Message>::new(match state {
                Pane::Graph => {
                    title_text = "Graph".to_string();
                    container(canvas(self.graph.clone()).width(Fill).height(Fill))
                        .padding(10)
                        .style(style::style::graph)
                }
                Pane::Text(t) => {
                    title_text = "About".to_string();
                    container(text(t))
                        .style(style::style::text)
                        .padding(10)
                        .width(Fill)
                        .height(Fill)
                }
                Pane::Slider => {
                    title_text = "Graph Controls".to_string();
                    container(
                        column!(
                            slider(0.0..=100.0, self.graph.x_shift, Message::XShift),
                            slider(0.0..=100.0, self.graph.y_shift, Message::YShift),
                            pick_list(
                                match &self.ports {
                                    Ok(ports) =>
                                        ports.iter().map(|port| port.port_name.clone()).collect(),
                                    Err(err) => vec![err.to_string()],
                                },
                                self.port.clone(),
                                Message::ChangePort
                            )
                            .placeholder("Choose a Port")
                            .width(Fill)
                            .padding(10),
                            button("redraw graph").on_press(Message::UpdateGraph)
                        )
                        .spacing(10),
                    )
                    .style(style::style::graph)
                    .padding(10)
                    .width(Fill)
                    .height(Fill)
                }
                Pane::Controls => {
                    title_text = "App Controls".to_string();

                    container(
                        button(text_input("Path", &self.path).on_input(Message::PathChanged))
                            .width(Fill)
                            .on_press(Message::Save),
                    )
                    .style(style::style::graph)
                    .width(Fill)
                    .height(Fill)
                    .padding(10)
                }
            })
            .title_bar(
                pane_grid::TitleBar::new(container(text(title_text)))
                    .style(style::style::title)
                    .padding(5),
            )
        })
        .spacing(10)
        .on_resize(10, Message::Resize)
        .on_drag(Message::Move);
        container(grid).style(style::style::app_s).padding(10)
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
            Message::Save => match csv::Writer::from_path(self.path.clone()) {
                Ok(mut wtr) => {
                    let _ = wtr.write_record(self.graph.values.iter().map(|v| format!("{}", v)));
                    let _ = wtr.flush();
                }
                _ => println!("invalid path"),
            },
            Message::PathChanged(path) => self.path = path,
            Message::ChangePort(port) => self.port = Some(port),
            Message::UpdateGraph => match &self.port {
                Some(p) => self.graph.values = read_port(p),
                None => {}
            },
        }
    }
}
fn function(x_size: usize) -> Vec<f32> {
    (0..x_size)
        .map(|x| ((x as f32 * 0.01).sin() + 1.0))
        .collect()
}
fn read_port(_: &String) -> Vec<f32> {
    //this is not going to be easy to implemnt and i would like to move to a new a file and import here
    //idk how todo this thought so i will need to look into it
    function(1000)
}
fn main() {
    let _ = iced::application("Graph", App::update, App::view).run();
}
