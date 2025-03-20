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
    avlb_ports: Vec<SerialPortInfo>,
    port: usize,
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
        let mut avlb_ports = serialport::available_ports().unwrap();
        avlb_ports.push(SerialPortInfo {
            port_name: "dummy".to_string(),
            port_type: serialport::SerialPortType::Unknown,
        });
        App {
            panes: g_state,
            path: "graph1.csv".to_string(),
            avlb_ports,
            port: 0,
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
                    controls_pane(&self.avlb_ports, self.port, self.path.clone(), pane)
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
            Message::ChangePort(port_name) => {
                let mut avlb_ports = serialport::available_ports().unwrap();
                avlb_ports.push(SerialPortInfo {
                    port_name: "dummy".to_string(),
                    port_type: serialport::SerialPortType::Unknown,
                });
                self.port = avlb_ports
                    .iter()
                    .position(|n| n.port_name == port_name)
                    .unwrap_or(0);
                self.avlb_ports = avlb_ports;
            }
            Message::Split(pane) => {
                self.panes.split(
                    pane_grid::Axis::Horizontal,
                    pane,
                    Pane::Graph(Graph::new(port::port::from_string(
                        &(self.avlb_ports[self.port].port_name),
                    ))),
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
                        .map(|(_, t)| {
                            if let Pane::Graph(g) = t {
                                if let Some(v) = g.port.next() {
                                    g.values.push(v)
                                }
                            }
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
    avlb_ports: &[SerialPortInfo],
    current_port: usize,
    path: String,
    pane: pane_grid::Pane,
) -> Container<Message> {
    container(
        column![
            row![
                pick_list(
                    avlb_ports
                        .iter()
                        .map(|port| port.port_name.clone())
                        .collect::<Vec<String>>(),
                    Some(avlb_ports[current_port].port_name.clone()),
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
fn write_file(data: Vec<&Vec<f32>>, path: &String) -> std::io::Result<()> {
    let mut f = fs::File::create(path)?;
    let max_size = data
        .get(data.len() - 1)
        .ok_or(std::io::Error::other("oh no"))?
        .len();
    println!("{}", max_size);
    for index in 0..max_size {
        writeln!(
            f,
            "{}",
            data.iter()
                .map(|vec| if let Some(x) = vec.get(max_size - index - 1) {
                    x.to_string() + ","
                } else {
                    ",".to_string()
                })
                .collect::<String>()
        )?
    }
    Ok(())
}
fn main() {
    let _ = iced::application("Graph", App::update, App::view)
        .subscription(App::subscription)
        .run();
}
