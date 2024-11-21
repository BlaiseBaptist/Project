pub mod port {
    use serialport;
    use std::fmt::Debug;
    use std::time::Duration;
    #[allow(dead_code)]
    pub trait Port: Debug + Iterator<Item = f32> {
        fn endian_value(&self) -> &str;
        fn swap_endianness(&mut self);
    }
    #[derive(Debug)]
    pub struct DummyPort {
        value_count: u32, //this is just to make it more interesting and clear what it is
    }
    impl Iterator for DummyPort {
        type Item = f32;
        #[allow(unused_mut)]
        fn next(&mut self) -> Option<Self::Item> {
            self.value_count += 1;
            Some((self.value_count as f32 / 100.0).sin() * 100.0)
        }
    }
    impl Port for DummyPort {
        fn endian_value(&self) -> &str {
            "not reading data"
        }
        fn swap_endianness(&mut self) {}
    }
    impl std::default::Default for DummyPort {
        fn default() -> Self {
            return DummyPort { value_count: 0 };
        }
    }
    #[derive(Debug)]
    pub struct PhysicalPort {
        pub port: Box<dyn serialport::SerialPort>,
        pub big_endian: bool,
    }
    impl Iterator for PhysicalPort {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            let mut serial_buf = [0b0 as u8; 4];
            match self.port.bytes_to_read().ok()? {
                4.. => {
                    self.port.read(&mut serial_buf).ok()?;
                }
                _ => {
                    return None;
                }
            };
            let value = if self.big_endian {
                u32::from_be_bytes(serial_buf)
            } else {
                u32::from_le_bytes(serial_buf)
            };
            Some(value as f32)
        }
    }
    impl Port for PhysicalPort {
        fn endian_value(&self) -> &str {
            return if self.big_endian {
                "big endian"
            } else {
                "little endian"
            };
        }
        fn swap_endianness(&mut self) {
            self.big_endian = !self.big_endian;
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
            Ok(v) => {
                println!("Success Opening Port");
                Box::new(v)
            }
            Err(e) => {
                println!("Error Opening {}: {} ( {:?} )", s, e.description, e.kind);
                Box::new(DummyPort::default())
            }
        }
    }
    fn try_open(s: &str) -> Result<impl Port, serialport::Error> {
        let port = serialport::new(s, 9600)
            .timeout(Duration::from_millis(100))
            .open()?;
        Ok(PhysicalPort {
            port,
            big_endian: true,
        })
    }
}
