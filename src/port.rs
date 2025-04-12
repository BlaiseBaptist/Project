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
    // #[derive(Debug)]
    // pub struct DummyPort {
    //     value_count: f32,
    // }
    // impl Iterator for DummyPort {
    //     type Item = Item;
    //     fn next(&mut self) -> Option<Self::Item> {
    //         if self.value_count > 100.0 {
    //             self.value_count = 1.0
    //         }
    //         self.value_count *= 1.0001;
    //         Some(self.value_count.to_be_bytes())
    //     }
    // }
    // impl Port for DummyPort {}
    // impl std::default::Default for DummyPort {
    //     fn default() -> Self {
    //         DummyPort { value_count: 1_f32 }
    //     }
    // }
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
                    println!("port is closed");
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
        let mut main_port = PhysicalPort::new(
            serialport::new(s, 9600)
                .open()
                .unwrap_or(Box::new(RealDummyPort::new())),
            internal_ports,
            s.to_string(),
        );
        let return_val = (0..internal_ports)
            .map(|_| main_port.split().unwrap())
            .collect();
        main_port.step_at(Duration::from_micros(1000));
        return return_val;
    }
    pub struct RealDummyPort {
        value: Item,
        value_count: usize,
    }
    impl RealDummyPort {
        fn new() -> Self {
            println!("creating dummy port");
            RealDummyPort {
                value: [0; 4],
                value_count: 0,
            }
        }
    }
    struct _AssertSend<T: Send>(T);
    impl _AssertSend<RealDummyPort> {}
    impl std::io::Read for RealDummyPort {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
            self.value_count += 1;
            self.value = ((self.value_count / 100) as f32).to_be_bytes();

            // self.value = (self.value_count as f32 / 100.0).sin().to_be_bytes();
            for x in 0..4 {
                buf[x] = self.value[x];
            }
            Ok(4)
        }
    }
    impl std::io::Write for RealDummyPort {
        fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
            for x in 0..4 {
                self.value[x] = buf[x]
            }
            Ok(4)
        }
        fn flush(&mut self) -> Result<(), std::io::Error> {
            todo!()
        }
    }
    impl serialport::SerialPort for RealDummyPort {
        fn name(&self) -> Option<String> {
            Some("dummy".to_string())
        }
        fn baud_rate(&self) -> Result<u32, serialport::Error> {
            Ok(0)
            //inf
        }
        fn data_bits(&self) -> Result<serialport::DataBits, serialport::Error> {
            Ok(serialport::DataBits::Eight)
        }
        fn flow_control(&self) -> Result<serialport::FlowControl, serialport::Error> {
            Ok(serialport::FlowControl::None)
        }
        fn parity(&self) -> Result<serialport::Parity, serialport::Error> {
            Ok(serialport::Parity::None)
        }
        fn stop_bits(&self) -> Result<serialport::StopBits, serialport::Error> {
            todo!()
        }
        fn timeout(&self) -> Duration {
            Duration::MAX
        }
        fn set_baud_rate(&mut self, _baud_rate: u32) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn set_data_bits(
            &mut self,
            _data_bits: serialport::DataBits,
        ) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn set_flow_control(
            &mut self,
            _flow_control: serialport::FlowControl,
        ) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn set_parity(&mut self, _parity: serialport::Parity) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn set_stop_bits(
            &mut self,
            _stop_bits: serialport::StopBits,
        ) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn set_timeout(&mut self, _timeout: Duration) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn write_request_to_send(&mut self, _level: bool) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn write_data_terminal_ready(&mut self, _level: bool) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn read_clear_to_send(&mut self) -> Result<bool, serialport::Error> {
            Ok(true)
        }
        fn read_data_set_ready(&mut self) -> Result<bool, serialport::Error> {
            Ok(true)
        }
        fn read_ring_indicator(&mut self) -> Result<bool, serialport::Error> {
            Ok(true)
        }
        fn read_carrier_detect(&mut self) -> Result<bool, serialport::Error> {
            Ok(true)
        }
        fn bytes_to_read(&self) -> Result<u32, serialport::Error> {
            Ok(1024)
            //inf
        }
        fn bytes_to_write(&self) -> Result<u32, serialport::Error> {
            Ok(4)
            //write only try to write the last value
        }
        fn clear(
            &self,
            _buffer_to_clear: serialport::ClearBuffer,
        ) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn try_clone(&self) -> Result<Box<dyn serialport::SerialPort>, serialport::Error> {
            Ok(Box::new(RealDummyPort::new()))
        }
        fn set_break(&self) -> Result<(), serialport::Error> {
            Ok(())
        }
        fn clear_break(&self) -> Result<(), serialport::Error> {
            Ok(())
        }
    }
}
