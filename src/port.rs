pub mod port {
    use serialport;
    use std::fmt::Debug;
    use std::time::Duration;
    type Item = f32;
    #[allow(dead_code)]
    pub trait Port: Debug + Iterator<Item = Item> {
        fn endian_value(&self) -> String;
        fn swap_endianness(&mut self);
    }
    #[derive(Debug)]
    pub struct DummyPort {
        value_count: u32, //this is just to make it more interesting
        dampen: f32,
    }
    impl Iterator for DummyPort {
        type Item = Item;
        #[allow(unused_mut)]
        fn next(&mut self) -> Option<Self::Item> {
            self.value_count += 1;
            self.dampen *= 1.0;
            let large_value = ((self.value_count as f32) / 100.0).sin();
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
            DummyPort {
                value_count: 0,
                dampen: 1.0,
            }
        }
    }
    #[derive(Debug)]
    pub struct PhysicalPort {
        pub port: Box<dyn serialport::SerialPort>,
        converter: converter,
        current_value: Item,
    }
    impl Iterator for PhysicalPort {
        type Item = Item;
        fn next(&mut self) -> Option<Self::Item> {
            let mut serial_buf = [0b0_u8; 4];
            match self.port.bytes_to_read().ok()? {
                4.. => {
                    self.port.read_exact(&mut serial_buf).ok()?;
                }
                _ => {
                    return None;
                }
            };
            let value = convert(&self.converter, serial_buf);
            self.current_value = value;
            Some(value)
        }
    }
    impl Port for PhysicalPort {
        fn endian_value(&self) -> String {
            match self.converter {
                converter::be_f32 => "be_f32",
                converter::le_f32 => "le_f32",
                converter::be_u32 => "be_u32",
                converter::le_u32 => "le_u32",
            }
            .to_string()
        }
        fn swap_endianness(&mut self) {
            self.converter = match self.converter {
                converter::be_f32 => converter::le_f32,
                converter::le_f32 => converter::be_u32,
                converter::be_u32 => converter::le_u32,
                converter::le_u32 => converter::be_f32,
            };
            println!("current value: {}", self.current_value);
        }
    }
    impl Clone for PhysicalPort {
        fn clone(&self) -> PhysicalPort {
            todo!()
        }
    }
    pub fn from_string(s: &str) -> Box<dyn Port<Item = Item>> {
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
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum converter {
        be_f32,
        le_f32,
        be_u32,
        le_u32,
    }
    fn convert(converter: &converter, data: [u8; 4]) -> Item {
        match converter {
            converter::be_f32 => f32::from_be_bytes(data),
            converter::le_f32 => f32::from_le_bytes(data),
            converter::be_u32 => u32::from_be_bytes(data) as f32,
            converter::le_u32 => u32::from_le_bytes(data) as f32,
        }
    }
    fn try_open(s: &str) -> Result<impl Port<Item = Item>, serialport::Error> {
        let port = serialport::new(s, 9600)
            .timeout(Duration::from_millis(100))
            .open()?;
        Ok(PhysicalPort {
            port,
            current_value: 0.0,
            converter: converter::be_f32,
        })
    }
}
