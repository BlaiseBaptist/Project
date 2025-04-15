use iced::{
    time,
    widget::{
        button, canvas, column, container, pane_grid, pane_grid::Configuration, pick_list, row,
        slider, text, text_input, toggler, Container,
    },
    Fill, Subscription,
};
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
    PathChanged(String),
    ChangeAvlbPort(String),
    ChangeOpenPort(String),
    OpenPort(usize, usize),
    ClosePort(usize),
    Split(pane_grid::Pane),
    Close(pane_grid::Pane),
    SwapEndianness(pane_grid::Pane),
    ChangeNumberOfPorts(usize),
    Save,
    Update,
    SwapTargetToSave(bool),
}
struct App {
    panes: pane_grid::State<Pane>,
    path: String,
    avlb_ports: Vec<String>,
    open_ports: Vec<Box<dyn port::port::Port>>,
    avlb_port: usize,
    open_port: usize,
    internal_ports: usize,
    open_delay: usize,
    is_buffer: bool,
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
        let open_ports = vec![];
        App {
            panes: g_state,
            path: "graph1.csv".to_string(),
            avlb_ports: get_avlb_ports(),
            open_ports,
            avlb_port: 0,
            open_port: 0,
            internal_ports: 1,
            open_delay: 0,
            is_buffer: true,
        }
    }
    fn view(&self) -> Container<Message> {
        let grid = pane_grid(&self.panes, |pane, state, _minimized| {
            let title_text: String;
            pane_grid::Content::<Message>::new(match state {
                Pane::Graph(g) => {
                    title_text = format!("graph: {}", g.port.name());
                    graph_pane(g, pane)
                }
                Pane::Controls => {
                    title_text = "App Controls".to_string();
                    controls_pane(
                        self.avlb_ports.clone(),
                        self.open_ports
                            .iter()
                            .map(|port| port.name())
                            .collect::<Vec<String>>(),
                        self.avlb_port,
                        self.open_port,
                        self.internal_ports,
                        self.path.clone(),
                        pane,
                        self.is_buffer,
                    )
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
                if self.is_buffer {
                    let _ = write_buffer(
                        self.panes
                            .iter()
                            .filter_map(|(_p, t)| match t {
                                Pane::Graph(g) => Some(g),
                                _ => None,
                            })
                            .collect(),
                    );
                    println!("saved to temp buffer");
                } else {
                    let _ = write_file(
                        self.panes
                            .iter()
                            .filter_map(|(_p, t)| match t {
                                Pane::Graph(g) => Some(g.get_values()),
                                _ => None,
                            })
                            .collect(),
                        &self.path,
                    );
                    println!("saved to file");
                }
            }
            Message::SwapTargetToSave(v) => self.is_buffer = v,
            Message::PathChanged(path) => self.path = path,
            Message::ChangeAvlbPort(port_name) => {
                self.avlb_ports = get_avlb_ports();
                self.avlb_port = self
                    .avlb_ports
                    .iter()
                    .position(|n| n == &port_name)
                    .unwrap_or(0);
            }
            Message::ChangeOpenPort(port_name) => {
                self.open_port = self
                    .open_ports
                    .iter()
                    .position(|n| n.name() == port_name)
                    .unwrap_or(0);
            }
            Message::OpenPort(port_index, number_of_ports) => {
                self.open_ports.append(&mut port::port::from_string(
                    self.avlb_ports[port_index].as_str(),
                    number_of_ports,
                ));
                if self.avlb_port >= self.avlb_ports.len() {
                    self.avlb_port = 0
                }
            }
            Message::ClosePort(port_index) => {
                if self.open_port == 0 {
                    return;
                }
                self.open_ports.remove(port_index);
                if self.open_port >= self.open_ports.len() {
                    self.open_port = 0
                }
            }
            Message::Split(pane) => {
                if self.open_port == 0 {
                    self.open_ports
                        .append(&mut port::port::from_string("dummy", 1))
                }
                self.panes.split(
                    pane_grid::Axis::Horizontal,
                    pane,
                    Pane::Graph(Graph::new(self.open_ports.remove(self.open_port))),
                );
                self.open_delay = 10;
                if self.open_port >= self.open_ports.len() {
                    self.open_port = 0
                }
            }
            Message::Close(pane) => {
                self.panes.close(pane);
            }
            Message::SwapEndianness(pane) => match self.panes.get_mut(pane) {
                Some(Pane::Graph(graph)) => graph.swap_endianness(),
                _ => unimplemented!(),
            },
            Message::ChangeNumberOfPorts(internal_ports) => self.internal_ports = internal_ports,
            Message::Update => {
                if self.open_delay == 0 {
                    let _: Vec<_> = self
                        .panes
                        .iter_mut()
                        .map(|(_, t)| {
                            if let Pane::Graph(g) = t {
                                if let Some(v) = g.port.next() {
                                    g.push(v)
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
        time::every(Duration::from_micros(100)).map(|_| Message::Update)
    }
}
fn controls_pane<'a>(
    avlb_ports: Vec<String>,
    open_ports: Vec<String>,
    current_avlb_port: usize,
    current_open_port: usize,
    internal_ports: usize,
    path: String,
    pane: pane_grid::Pane,
    is_buffer: bool,
) -> Container<'a, Message> {
    let avlb_port = avlb_ports[current_avlb_port].clone();
    let open_port = open_ports
        .get(current_open_port)
        .map_or("".to_string(), |v| v.clone());
    container(
        column![
            row![
                pick_list(avlb_ports, Some(avlb_port), Message::ChangeAvlbPort),
                button(text(format!("Open {} Ports", internal_ports)))
                    .on_press(Message::OpenPort(current_avlb_port, internal_ports)),
                slider(1_f32..=32_f32, internal_ports as f32, |x| {
                    Message::ChangeNumberOfPorts(x as usize)
                })
            ]
            .spacing(10),
            row![
                pick_list(open_ports, Some(open_port), Message::ChangeOpenPort),
                button("New Graph").on_press(Message::Split(pane)),
                button("Close Port").on_press(Message::ClosePort(current_open_port))
            ]
            .spacing(10),
            toggler(is_buffer)
                .on_toggle(Message::SwapTargetToSave)
                .label("Save to Temp Buffer"),
            button(
                row![
                    text("save to:")
                        .align_y(iced::alignment::Vertical::Center)
                        .line_height(1.5)
                        .height(30),
                    text_input("Path", &path)
                        .on_input_maybe(if is_buffer {
                            None
                        } else {
                            Some(Message::PathChanged)
                        })
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
                graph.converter.name()
            )))
            .on_press(Message::SwapEndianness(pane))
        ]
        .spacing(10)
    ])
    .padding(10)
    .style(style::style::graph)
}
fn write_buffer(data: Vec<&Graph>) -> std::io::Result<()> {
    for graph in data {
        let mut f = fs::File::create(format!(".buffer{}", graph.port.name()))?;
        let _ = f.write_all(graph.values.as_flattened())?;
    }
    Ok(())
}
fn get_avlb_ports() -> Vec<String> {
    serialport::available_ports()
        .unwrap()
        .into_iter()
        .map(|port| port.port_name)
        .chain(vec!["dummy".to_string(), ".buffer".to_string()].into_iter())
        .collect()
}
fn write_file(data: Vec<Vec<f32>>, path: &String) -> std::io::Result<()> {
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
