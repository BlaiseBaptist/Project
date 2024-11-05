pub mod port {
    #[derive(Default)]
	pub struct Port {
        pub current_value: f32,
    }
    impl Iterator for Port {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            //read port here
            Some(10.0)
        }
    }
}
