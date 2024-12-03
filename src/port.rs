pub mod port {
    use serialport;
    use std::fmt::Debug;
    use std::time::Duration;
    #[allow(dead_code)]
    pub trait Port: Debug + Iterator<Item = u32> {
        fn endian_value(&self) -> String;
        fn swap_endianness(&mut self);
    }
    #[derive(Debug)]
    pub struct DummyPort {
        value_count: u32, //this is just to make it more interesting
        dampen: f32,
    }
    impl Iterator for DummyPort {
        type Item = u32;
        #[allow(unused_mut)]
        fn next(&mut self) -> Option<Self::Item> {
            self.value_count += 1;
            self.dampen *= 1.0;
            let large_value = (2u32.pow(31u32) / 1) * (self.value_count % 2);
            Some(large_value)
        }
    }
    impl Port for DummyPort {
        fn endian_value(&self) -> String {
            format!("has {} elements", self.value_count)
        }
        fn swap_endianness(&mut self) {}
    }
    impl std::default::Default for DummyPort {
        fn default() -> Self {
            return DummyPort {
                value_count: 0,
                dampen: 1.0,
            };
        }
    }
    #[derive(Debug)]
    pub struct PhysicalPort {
        pub port: Box<dyn serialport::SerialPort>,
        pub big_endian: bool,
        current_value: u32,
    }
    impl Iterator for PhysicalPort {
        type Item = u32;
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
            self.current_value = value;
            Some(value)
        }
    }
    impl Port for PhysicalPort {
        fn endian_value(&self) -> String {
            return (if self.big_endian {
                "big endian"
            } else {
                "little endian"
            })
            .to_string();
        }
        fn swap_endianness(&mut self) {
            println!("current value: {}", self.current_value);
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
            current_value: 0,
            big_endian: true,
        })
    }
}
