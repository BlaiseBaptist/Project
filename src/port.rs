pub mod port {
    use std::fmt::Debug;

    use serialport;

    pub trait Port: Debug {
        fn next(&mut self) -> Option<f32>;
        fn value(&self) -> f32;
    }

    pub fn default_port() -> Box<dyn Port> {
        return Box::new(DummyPort{current_value: 1.0})
    }

    pub fn from_string(s: &str) -> Result<Box<dyn Port>, serialport::Error> {
        println!("Parsing {}", s);
        if s == "dummy" {
            return Ok(Box::new(DummyPort{current_value: 1.0}))
        }
        let mut port = serialport::new(s, 9600).open()?;
        let mut serial_buf: Vec<u8> = vec![0; 32];
        Ok(Box::new(PhysicalPort {
            current_value: port.read(serial_buf.as_mut_slice())? as f32,
            port,
        }))
    }

    #[derive(Debug)]
    pub struct DummyPort {
        pub current_value: f32,
    }

    impl Port for DummyPort {
        fn next(&mut self) -> Option<f32> {
            Some(self.current_value)
        }
        fn value(&self) -> f32 {
            self.current_value
        }
    }

    #[derive(Debug)]
    pub struct PhysicalPort {
        pub current_value: f32,
        pub port: Box<dyn serialport::SerialPort>,
    }
    impl Port for PhysicalPort {
        fn next(&mut self) -> Option<f32> {
            let mut serial_buf: Vec<u8> = vec![0; 32];
            Some(self.port.read(serial_buf.as_mut_slice()).ok()? as f32)
        }
        fn value(&self) -> f32 {
            return self.current_value;
        }
    }
    impl std::str::FromStr for PhysicalPort {
        type Err = serialport::Error;
        fn from_str(string: &str) -> Result<Self, Self::Err> {
            let mut port = serialport::new(string, 9600).open()?;
            let mut serial_buf: Vec<u8> = vec![0; 32];
            Ok(PhysicalPort {
                current_value: port.read(serial_buf.as_mut_slice())? as f32,
                port,
            })
        }
    }
    impl std::default::Default for PhysicalPort {
        fn default() -> Self {
            let ports = serialport::available_ports().expect("No ports found!");
            ports.iter().find_map(try_open).unwrap()
        }
    }
    impl Clone for PhysicalPort {
        fn clone(&self) -> PhysicalPort {
            todo!()
        }
    }
    fn try_open(port: &serialport::SerialPortInfo) -> Option<PhysicalPort> {
        println!("{:?}", port.port_name);
        port.port_name.parse().ok()
    }
}
