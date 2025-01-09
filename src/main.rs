use iced::{
    time,
    widget::{
        button, canvas, column, container, pane_grid, pane_grid::Configuration, pick_list, row,
        text, text_input, Container,
    },
    Fill, Subscription,
};
use serialport::SerialPortInfo;
use std::{fs, io::Write, time::Duration};
mod graph;
mod port;
mod style;
use graph::graph::Graph;
enum Pane {
    Graph(Graph),
    Controls,
}
#[derive(Debug, Clone)]
enum Message {
    Resize(pane_grid::ResizeEvent),
    Move(pane_grid::DragEvent),
    Save,
    PathChanged(String),
    ChangePort(String),
    Split(pane_grid::Pane),
    Close(pane_grid::Pane),
    SwapEndianness(pane_grid::Pane),
    Update,
}
struct App {
    panes: pane_grid::State<Pane>,
    path: String,
    ports: Vec<SerialPortInfo>,
    port: String,
    open_delay: usize,
}
impl Default for App {
    fn default() -> App {
        App::new()
    }
}
impl App {
    fn new() -> Self {
        let config = Configuration::Pane(Pane::Controls);
        let g_state = pane_grid::State::with_configuration(config);
        let mut ports = serialport::available_ports().unwrap();
        ports.push(SerialPortInfo {
            port_name: "dummy".to_string(),
            port_type: serialport::SerialPortType::Unknown,
        });
        App {
            panes: g_state,
            path: "graph1.csv".to_string(),
            ports: ports,
            port: "dummy".to_string(),
            open_delay: 0,
        }
    }
    fn view(&self) -> Container<Message> {
        let grid = pane_grid(&self.panes, |pane, state, _minimized| {
            let title_text: String;
            pane_grid::Content::<Message>::new(match state {
                Pane::Graph(g) => {
                    title_text = "Graph".to_string();
                    graph_pane(g, pane)
                }
                Pane::Controls => {
                    title_text = "App Controls".to_string();
                    controls_pane(&self.ports, self.port.clone(), self.path.clone(), pane)
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
            Message::Save => {
                let _ = write_file(
                    self.panes
                        .iter()
                        .filter_map(|(_p, t)| match t {
                            Pane::Graph(g) => Some(&g.values),
                            _ => None,
                        })
                        .collect(),
                    &self.path,
                );
            }
            Message::PathChanged(path) => self.path = path,
            Message::ChangePort(port) => {
                let mut ports = serialport::available_ports().unwrap();
                ports.push(SerialPortInfo {
                    port_name: "dummy".to_string(),
                    port_type: serialport::SerialPortType::Unknown,
                });
                self.ports = ports;
                self.port = port;
            }
            Message::Split(pane) => {
                self.panes.split(
                    pane_grid::Axis::Horizontal,
                    pane,
                    Pane::Graph(Graph::new(port::port::from_string(&self.port))),
                );
                self.open_delay = 10;
            }
            Message::Close(pane) => {
                self.panes.close(pane);
            }
            Message::SwapEndianness(pane) => match self.panes.get_mut(pane) {
                Some(Pane::Graph(graph)) => graph.swap_endianness(),
                _ => unimplemented!(),
            },
            Message::Update => {
                if self.open_delay == 0 {
                    let _: Vec<_> = self
                        .panes
                        .iter_mut()
                        .map(|(_, t)| match t {
                            Pane::Graph(g) => match g.port.next() {
                                Some(v) => g.values.push(v),
                                None => {}
                            },
                            _ => {}
                        })
                        .collect();
                } else {
                    self.open_delay -= 1
                }
            }
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_nanos(1)).map(|_| Message::Update)
    }
}
fn controls_pane(
    ports: &[SerialPortInfo],
    current_port: String,
    path: String,
    pane: pane_grid::Pane,
) -> Container<Message> {
    container(
        column![
            row![
                pick_list(
                    ports
                        .iter()
                        .map(|port| port.port_name.clone())
                        .collect::<Vec<_>>(),
                    Some(current_port),
                    Message::ChangePort
                ),
                button("New Graph").on_press(Message::Split(pane))
            ]
            .spacing(10),
            button(
                row![
                    text("save to:")
                        .align_y(iced::alignment::Vertical::Center)
                        .line_height(1.5)
                        .height(30),
                    text_input("Path", &path)
                        .on_input(Message::PathChanged)
                        .line_height(1.5)
                ]
                .spacing(10)
            )
            .width(Fill)
            .on_press(Message::Save),
        ]
        .spacing(10),
    )
    .style(style::style::graph)
    .width(Fill)
    .height(Fill)
    .padding(10)
}
fn graph_pane(graph: &Graph, pane: pane_grid::Pane) -> Container<Message> {
    container(column![
        canvas(graph).width(Fill).height(Fill),
        row![
            button("Close Pane").on_press(Message::Close(pane)),
            button(text(format!(
                "Swap Endianness (currently {})",
                graph.port.endian_value()
            )))
            .on_press(Message::SwapEndianness(pane))
        ]
        .spacing(10)
    ])
    .padding(10)
    .style(style::style::graph)
}
fn write_file(data: Vec<&Vec<u32>>, path: &String) -> std::io::Result<()> {
    let mut f = fs::File::create(path)?;
    f.write_all(
        data.iter()
            .map(|vec| {
                vec.iter()
                    .map(|val| val.to_string() + ",")
                    .collect::<String>()
                    + "\n"
            })
            .collect::<String>()
            .as_bytes(),
    )?;
    Ok(())
}
fn main() {
    let _ = iced::application("Graph", App::update, App::view)
        .subscription(App::subscription)
        .run();
}
