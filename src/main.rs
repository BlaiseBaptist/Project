use iced::{
    time,
    widget::{
        button, canvas, column, container, pane_grid, pane_grid::Configuration, pick_list, row,
        slider, text, text_input, Button, Container, Space,
    },
    Fill, Subscription,
};
use std::{
    fs,
    io::{Read, Write},
    time::Duration,
};
mod graph;
mod port;
mod style;
use graph::graph::Graph;
use port::port::from_string;
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
    Save(bool),
    OpenBuffer(pane_grid::Pane),
    Update,
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
    status_message: String,
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
            status_message: "Ready to go".to_string(),
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
                    title_text = format!("Status: {:>2}", self.status_message);
                    controls_pane(
                        self.avlb_ports.clone(),
                        self.open_ports
                            .iter()
                            .map(|port| port.name())
                            .collect::<Vec<String>>(),
                        self.avlb_port,
                        self.open_port,
                        self.internal_ports,
                        &self.path,
                        pane,
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
            Message::Save(is_buffer) => {
                if is_buffer {
                    let _ = write_buffer(
                        self.panes
                            .iter()
                            .filter_map(|(_p, t)| match t {
                                Pane::Graph(g) => Some(g),
                                _ => None,
                            })
                            .collect(),
                    );
                    self.status_message = "Saved All Graphs to Buffer".to_string();
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
                    self.status_message = "Saved All Graphs to File".to_string();
                }
            }
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
                self.status_message = format!("Opening {}", self.avlb_ports[port_index]);
                self.open_ports.append(&mut from_string(
                    self.avlb_ports[port_index].as_str(),
                    number_of_ports,
                    None,
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
                if self.open_port >= self.open_ports.len() {
                    return;
                }
                self.panes.split(
                    pane_grid::Axis::Horizontal,
                    pane,
                    Pane::Graph(Graph::new(self.open_ports.remove(self.open_port))),
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
            Message::ChangeNumberOfPorts(internal_ports) => self.internal_ports = internal_ports,
            Message::OpenBuffer(pane) => {
                let mut file = fs::File::open(".buffer").expect("no buffer");
                let mut buf: Vec<u8> = Vec::new();
                file.read_to_end(&mut buf).expect("how did you get here? ");
                let values: Vec<[u8; 4]> = buf
                    .as_slice()
                    .chunks_exact(4)
                    .map(|x| x.try_into().unwrap())
                    .collect();
                for buf_port in from_string("buffer", self.internal_ports, Some(&values)) {
                    self.panes
                        .split(
                            pane_grid::Axis::Horizontal,
                            pane,
                            Pane::Graph(Graph::new(buf_port)),
                        )
                        .expect("how did you get here?");
                }
                self.status_message = "Open Graph From buffer".to_string();
                self.open_delay = 10;
            }
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
const LINE_HEIGHT: f32 = 1.6;
const TEXT_SIZE: f32 = 16.0;
const UNIT_WIDTH: f32 = 160.0;
const ROW_SPACING: f32 = 8.0;
const ROW_HEIGHT: f32 = 32.0;
fn controls_pane<'a>(
    avlb_ports: Vec<String>,
    open_ports: Vec<String>,
    current_avlb_port: usize,
    current_open_port: usize,
    internal_ports: usize,
    path: &str,
    pane: pane_grid::Pane,
) -> Container<'a, Message> {
    let avlb_port = avlb_ports[current_avlb_port].clone();
    let open_port = open_ports
        .get(current_open_port)
        .map_or("".to_string(), |v| v.clone());
    container(
        column![
            row![
                controls_pane_button(Box::leak::<'a>(
                    format!(
                        "Open {} Port{}",
                        internal_ports,
                        if internal_ports == 1 { "" } else { "s" }
                    )
                    .into_boxed_str()
                ))
                .on_press(Message::OpenPort(current_avlb_port, internal_ports)),
                controls_pane_button("New Graph").on_press(Message::Split(pane)),
                controls_pane_button("Close Port").on_press(Message::ClosePort(current_open_port)),
                Space::with_width(Fill),
                pick_list(open_ports, Some(open_port), Message::ChangeOpenPort)
                    .text_line_height(LINE_HEIGHT)
                    .text_size(TEXT_SIZE)
                    .width(UNIT_WIDTH * 3.0),
            ]
            .height(ROW_HEIGHT)
            .spacing(ROW_SPACING)
            .align_y(iced::alignment::Vertical::Center),
            row![
                slider(1_f32..=32_f32, internal_ports as f32, |x| {
                    Message::ChangeNumberOfPorts(x as usize)
                })
                .width(UNIT_WIDTH * 3.0 + ROW_SPACING * 2.0),
                Space::with_width(Fill),
                pick_list(avlb_ports, Some(avlb_port), Message::ChangeAvlbPort)
                    .text_line_height(LINE_HEIGHT)
                    .text_size(TEXT_SIZE)
                    .width(UNIT_WIDTH * 3.0),
            ]
            .height(ROW_HEIGHT)
            .spacing(ROW_SPACING)
            .align_y(iced::alignment::Vertical::Center),
            row![
                controls_pane_button(Box::leak::<'a>(
                    format!(
                        "Open {} Buffer{}",
                        internal_ports,
                        if internal_ports == 1 { "" } else { "s" }
                    )
                    .into_boxed_str()
                ))
                .on_press(Message::OpenBuffer(pane)),
                controls_pane_button("Save to Buffer").on_press(Message::Save(true)),
                controls_pane_button("Save to:").on_press(Message::Save(false)),
                Space::with_width(Fill),
                text_input("Path", &path)
                    .on_input(Message::PathChanged)
                    .on_submit(Message::Save(false))
                    .line_height(LINE_HEIGHT)
                    .size(TEXT_SIZE)
                    .width(UNIT_WIDTH * 3.0)
            ]
            .height(ROW_HEIGHT)
            .spacing(ROW_SPACING)
            .align_y(iced::alignment::Vertical::Center),
        ]
        .spacing(8.0), //column spacing
    )
    .style(style::style::graph)
    .width(Fill)
    .height(Fill)
    .padding(ROW_SPACING)
}
fn controls_pane_button<'a>(contents: &'a str) -> Button<'a, Message> {
    button(
        text(contents)
            .line_height(LINE_HEIGHT)
            .size(TEXT_SIZE)
            .center(),
    )
    .width(UNIT_WIDTH)
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
        println!("{:?}", graph);
        let mut file = fs::File::create(".buffer")?;
        file.write_all(graph.values.as_flattened())?;
        file.flush()?;
    }
    Ok(())
}
fn get_avlb_ports() -> Vec<String> {
    vec!["dummy".to_string()]
        .into_iter()
        .chain(
            serialport::available_ports()
                .unwrap()
                .into_iter()
                .map(|port| port.port_name),
        )
        .collect()
}
fn write_file(data: Vec<Vec<f32>>, path: &String) -> std::io::Result<()> {
    let mut f = fs::File::create(path)?;
    let max_size = data
        .get(data.len() - 1)
        .ok_or(std::io::Error::other("oh no"))?
        .len();
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
