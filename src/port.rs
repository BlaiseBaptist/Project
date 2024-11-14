pub mod port {
    use serialport;
    use std::fmt::Debug;
    use std::time::Duration;
    #[allow(dead_code)]
    pub trait Port: Debug + Iterator<Item = f32> {
        fn value(&self) -> f32;
    }
    #[derive(Debug)]
    pub struct DummyPort {
        pub current_value: f32,
        value_count: u32, //this is just to make it more interesting and clear what it is
    }
    impl Iterator for DummyPort {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            self.value_count += 1;
            Some(((self.value_count as f32) / 100.0).sin() * 100.0)
        }
    }
    impl Port for DummyPort {
        fn value(&self) -> f32 {
            self.current_value
        }
    }
    impl std::default::Default for DummyPort {
        fn default() -> Self {
            return DummyPort {
                current_value: 1.0,
                value_count: 0,
            };
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
            let mut bytes: Vec<u8> = Vec::new();
            while bytes.len() < 32 {
                let mut serial_buf: Vec<u8> = vec![0];
                let _ = self.port.read(serial_buf.as_mut_slice()).ok()?;
                bytes.push(serial_buf[0]);
            }
            //println!("#bytes {}, bytes {:?}", bytes.len(), bytes);
            let value = bytes.iter().enumerate().reduce(|(acc, _), (p, b)| {
                (acc + (*b as usize) * 2_usize.pow(p.try_into().unwrap()), &1)
            })?;
            Some(value.0 as f32)
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
            return Box::new(DummyPort::default());
        };
        match try_open(s) {
            Ok(v) => Box::new(v),
            Err(e) => {
                println!("Error Opening {}: {} ( {:?} )", s, e.description, e.kind);
                Box::new(DummyPort::default())
            }
        }
    }
    fn try_open(s: &str) -> Result<impl Port, serialport::Error> {
        let mut port = serialport::new(s, 9600)
            .timeout(Duration::from_millis(100))
            .open()?;
        let mut serial_buf: Vec<u8> = vec![0; 32];
        Ok(PhysicalPort {
            current_value: port.read(serial_buf.as_mut_slice())? as f32,
            port,
        })
    }
}
