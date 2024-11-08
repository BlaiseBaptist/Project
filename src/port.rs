pub mod port {
    use serialport;
    #[derive(Debug)]
    pub struct Port {
        pub current_value: f32,
        pub port: Box<dyn serialport::SerialPort>,
    }
    impl Iterator for Port {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            let mut serial_buf: Vec<u8> = vec![0; 32];
            Some(self.port.read(serial_buf.as_mut_slice()).ok()? as f32)
        }
    }
    impl std::str::FromStr for Port {
        type Err = serialport::Error;
        fn from_str(string: &str) -> Result<Self, Self::Err> {
            let mut port = serialport::new(string, 9600).open()?;
            let mut serial_buf: Vec<u8> = vec![0; 32];
            Ok(Port {
                current_value: port.read(serial_buf.as_mut_slice())? as f32,
                port,
            })
        }
    }
    impl std::default::Default for Port {
        fn default() -> Self {
            let ports = serialport::available_ports().expect("No ports found!");
            ports.iter().find_map(try_open).unwrap()
        }
    }
    impl Clone for Port {
        fn clone(&self) -> Port {
            todo!()
        }
    }
    fn try_open(port: &serialport::SerialPortInfo) -> Option<Port> {
        println!("{:?}", port.port_name);
        port.port_name.parse().ok()
    }
}
