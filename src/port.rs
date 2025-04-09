pub mod port {
    use serialport;
    use std::fmt::Debug;
    use std::sync::mpsc;
    use std::time::Duration;
    type Item = [u8; 4];
    #[allow(dead_code)]
    pub trait Port: Debug + Iterator<Item = Item> {
        fn name(&self) -> String {
            "dummy".to_string()
        }
    }
    #[derive(Debug)]
    pub struct DummyPort {
        value_count: f32,
    }
    impl Iterator for DummyPort {
        type Item = Item;
        fn next(&mut self) -> Option<Self::Item> {
            if self.value_count > 100.0 {
                self.value_count = 1.0
            }
            self.value_count *= 1.0001;
            Some(self.value_count.to_be_bytes())
        }
    }
    impl Port for DummyPort {}
    impl std::default::Default for DummyPort {
        fn default() -> Self {
            DummyPort { value_count: 1_f32 }
        }
    }
    #[derive(Debug)]
    struct MultiPort {
        port: mpsc::Receiver<Item>,
        name: String,
    }
    impl Iterator for MultiPort {
        type Item = Item;
        fn next(&mut self) -> Option<Self::Item> {
            self.port.try_recv().ok()
        }
    }
    impl Port for MultiPort {
        fn name(&self) -> String {
            self.name.clone()
        }
    }
    #[derive(Debug)]
    struct PhysicalPort {
        port: Box<dyn serialport::SerialPort>,
        name: String,
        values: Vec<(mpsc::Sender<Item>, Option<mpsc::Receiver<Item>>)>,
        internal_ports: usize,
        current_port_read: usize,
    }
    impl PhysicalPort {
        fn new(port: Box<dyn serialport::SerialPort>, internal_ports: usize, name: String) -> Self {
            let mut values = Vec::with_capacity(internal_ports);
            for _ in 0..internal_ports {
                let (sender, receiver) = mpsc::channel::<Item>();
                values.push((sender, Some(receiver)));
            }
            PhysicalPort {
                port,
                name,
                values,
                internal_ports,
                current_port_read: 0,
            }
        }
        fn split(&mut self) -> Option<Box<dyn Port>> {
            self.current_port_read += 1;
            Some(Box::new(MultiPort {
                port: self.values.get_mut(self.current_port_read - 1)?.1.take()?,
                name: format!("{} split {}", self.name.clone(), self.current_port_read),
            }))
        }
        fn step_at(mut self, time: Duration) {
            std::thread::spawn(move || loop {
                if !self.next() {
                    return;
                }
                std::thread::sleep(time);
            });
        }
        fn next(&mut self) -> bool {
            let mut serial_buf = vec![[0b0_u8; 4]; self.internal_ports];
            match self.port.bytes_to_read() {
                Ok(v) if v as usize >= self.internal_ports * 4 => {
                    for x in 0..serial_buf.len() {
                        let _ = self.port.read_exact(&mut serial_buf[x]);
                        if self.values[x].0.send(serial_buf[x]).is_err() {
                            return false;
                        }
                    }
                    true
                }
                _ => true,
            }
        }
    }
    pub fn from_string(s: &str, internal_ports: usize) -> Vec<Box<dyn Port>> {
        if s == "dummy" {
            return vec![Box::new(DummyPort::default())];
        };
        match serialport::new(s, 9600)
            .timeout(Duration::from_millis(100))
            .open()
        {
            Ok(v) => {
                println!("Success opening port. Splitting {} times", internal_ports);
                let mut main_port = PhysicalPort::new(v, internal_ports, s.to_string());
                let return_val = (0..internal_ports)
                    .map(|_| main_port.split().unwrap_or(Box::new(DummyPort::default())))
                    .collect();
                main_port.step_at(Duration::from_secs(1));
                return_val
            }
            Err(e) => {
                println!("Error Opening {}: {} ( {:?} )", s, e.description, e.kind);
                vec![Box::new(DummyPort::default())]
            }
        }
    }
}
