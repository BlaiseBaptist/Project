pub mod port {
    use serialport;
    use std::fmt::Debug;
    pub trait Port: Debug + Iterator<Item = f32> {
        fn value(&self) -> f32;
    }
    #[derive(Debug)]
    pub struct DummyPort {
        pub current_value: f32,
    }
    impl Iterator for DummyPort {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            Some(self.current_value)
        }
    }
    impl Port for DummyPort {
        fn value(&self) -> f32 {
            self.current_value
        }
    }
    impl std::default::Default for DummyPort {
        fn default() -> Self {
            return DummyPort { current_value: 1.0 };
        }
    }
    #[derive(Debug)]
    pub struct PhysicalPort {
        pub current_value: f32,
        pub port: Box<dyn serialport::SerialPort>,
    }
    impl Iterator for PhysicalPort {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            let mut serial_buf: Vec<u8> = vec![0; 32];
            Some(self.port.read(serial_buf.as_mut_slice()).ok()? as f32)
        }
    }
    impl Port for PhysicalPort {
        fn value(&self) -> f32 {
            return self.current_value;
        }
    }
    impl Clone for PhysicalPort {
        fn clone(&self) -> PhysicalPort {
            todo!()
        }
    }
    pub fn from_string(s: &str) -> Box<dyn Port> {
        if s == "dummy" {
            return Box::new(DummyPort { current_value: 1.0 });
        };
        match try_open(s) {
            Ok(v) => v,
            Err(_) => Box::new(DummyPort { current_value: 1.0 }),
        }
    }
    fn try_open(s: &str) -> Result<Box<dyn Port>, serialport::Error> {
        let mut port = serialport::new(s, 9600).open()?;
        let mut serial_buf: Vec<u8> = vec![0; 32];
        Ok(Box::new(PhysicalPort {
            current_value: port.read(serial_buf.as_mut_slice())? as f32,
            port,
        }))
    }
}
