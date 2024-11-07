pub mod port {
    #[derive(Default, Clone, Debug)]
    pub struct Port {
        pub current_value: f32,
    }
    impl Iterator for Port {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            //read port here
            Some(self.current_value)
        }
    }
    impl From<String> for Port {
        fn from(_string: String) -> Self {
            Port {
                current_value: 10.0,
            }
        }
    }
}
