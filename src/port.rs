pub mod port {
    use serialport;
    use std::fmt::Debug;
    use std::sync::mpsc;
    use std::time::Duration;
    type Item = f32;
    #[allow(dead_code)]
    pub trait Port: Debug + Iterator<Item = Item> {
        fn endian_value(&self) -> String {
            self.converter()
                .map(|c| c.name())
                .unwrap_or("None".to_string())
        }
        fn swap_endianness(&mut self) {
            self.converter().map(|c| c.swap());
        }
        fn split(&mut self) -> Option<Box<dyn Port>> {
            Some(Box::new(DummyPort::default()))
        }
        fn converter(&self) -> Option<converter> {
            None
        }
        fn name(&self) -> String {
            "dummy".to_string()
        }
    }
    #[derive(Debug)]
    pub struct DummyPort {
        value_count: u32,
        dampen: f32,
    }
    impl Iterator for DummyPort {
        type Item = Item;
        #[allow(unused_mut)]
        fn next(&mut self) -> Option<Self::Item> {
            self.value_count += 1;
            let value = (10000.0 / self.value_count as f32).sin() * self.dampen;
            self.dampen *= 0.99;
            if self.dampen < self.value_count as f32 {
                self.dampen *= 1.1;
            }
            Some(value)
        }
    }
    impl Port for DummyPort {}
    impl std::default::Default for DummyPort {
        fn default() -> Self {
            DummyPort {
                value_count: 0,
                dampen: 1.0,
            }
        }
    }
    #[derive(Debug)]
    struct MultiPort {
        port: mpsc::Receiver<Item>,
    }
    impl Iterator for MultiPort {
        type Item = Item;
        fn next(&mut self) -> Option<Self::Item> {
            self.port.try_recv().ok()
        }
    }
    impl Port for MultiPort {
        fn name(&self) -> String {
            "MultiPort".to_string()
        }
    }
    #[derive(Debug)]
    pub struct PhysicalPort {
        pub port: Box<dyn serialport::SerialPort>,
        name: String,
        values: Vec<(mpsc::Sender<Item>, Option<mpsc::Receiver<Item>>)>,
        internal_ports: usize,
        current_port_write: usize,
        current_port_read: usize,
        converter: converter,
    }
    impl PhysicalPort {
        fn new(
            port: Box<dyn serialport::SerialPort>,
            internal_ports: usize,
            converter: Option<converter>,
            name: String,
        ) -> Self {
            let mut values = Vec::with_capacity(internal_ports);
            for _ in 0..internal_ports {
                let (sender, receiver) = mpsc::channel::<Item>();
                values.push((sender, Some(receiver)));
            }
            PhysicalPort {
                port,
                name,
                values,
                internal_ports: internal_ports,
                current_port_write: 0,
                current_port_read: 0,
                converter: converter.unwrap_or(converter::be_f32),
            }
        }
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
            let value = self.converter.convert(serial_buf);
            let _ = self.values[self.current_port_write].0.send(value);
            if self.current_port_write < self.internal_ports {
                self.current_port_write += 1;
            } else {
                self.current_port_write = 0;
            }
            Some(value)
        }
    }
    impl Port for PhysicalPort {
        fn split(&mut self) -> Option<Box<dyn Port>> {
            Some(Box::new(MultiPort {
                port: self.values[self.current_port_read].1.take()?,
            }))
        }
        fn name(&self) -> String {
            self.name.clone()
        }
    }
    pub fn from_string(s: &str, internal_ports: usize) -> Box<dyn Port<Item = Item>> {
        if s == "dummy" {
            return Box::new(DummyPort::default());
        };
        match serialport::new(s, 9600)
            .timeout(Duration::from_millis(100))
            .open()
        {
            Ok(v) => {
                println!("Success Opening Port");
                Box::new(PhysicalPort::new(v, internal_ports, None, s.to_string()))
            }
            Err(e) => {
                println!("Error Opening {}: {} ( {:?} )", s, e.description, e.kind);
                Box::new(DummyPort::default())
            }
        }
    }
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq)]
    pub enum converter {
        be_f32,
        le_f32,
        be_u32,
        le_u32,
        u8_to_string,
    }
    impl converter {
        fn convert(&self, data: [u8; 4]) -> Item {
            match self {
                converter::be_f32 => f32::from_be_bytes(data),
                converter::le_f32 => f32::from_le_bytes(data),
                converter::be_u32 => u32::from_be_bytes(data) as f32,
                converter::le_u32 => u32::from_le_bytes(data) as f32,
                converter::u8_to_string => data
                    .into_iter()
                    .map(|b| char::from(b))
                    .collect::<String>()
                    .parse::<f32>()
                    .unwrap_or(0.0),
            }
        }
        fn swap(&self) -> Self {
            match self {
                converter::be_f32 => converter::le_f32,
                converter::le_f32 => converter::be_u32,
                converter::be_u32 => converter::le_u32,
                converter::le_u32 => converter::u8_to_string,
                converter::u8_to_string => converter::be_f32,
            }
        }
        fn name(&self) -> String {
            match self {
                converter::be_f32 => "be_f32",
                converter::le_f32 => "le_f32",
                converter::be_u32 => "be_u32",
                converter::le_u32 => "le_u32",
                converter::u8_to_string => "u8_to_string",
            }
            .to_string()
        }
    }
}
