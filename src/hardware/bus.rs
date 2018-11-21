pub trait Bus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

// A bus which reads the specified bytes in sequence
#[cfg(test)]
pub mod seq {
    use std::cell::Cell;

    #[derive(Debug, Clone)]
    pub struct Bus {
        counter: Cell<usize>,
        pub bytes: Vec<u8>,
    }

    impl Bus {
        pub fn new(bytes: Vec<u8>) -> Self { Self { counter: Cell::new(0), bytes } }
    }

    impl PartialEq for Bus {
        fn eq(&self, _other: &Self) -> bool { true }
    }

    impl super::Bus for Bus {
        fn read(&self, _addr: u16) -> u8 {
            let counter = self.counter.get();
            let res = self.bytes[counter % self.bytes.len()];
            self.counter.set(counter + 1);
            res
        }
        fn write(&mut self, _addr: u16, _data: u8) {}
    }
}
