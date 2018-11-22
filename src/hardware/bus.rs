pub trait Bus {
    // Reading a Bus on the NES can have side effects, so it'll be mutable here.
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

// A bus which reads the specified bytes in sequence
#[cfg(test)]
pub mod seq {
    #[derive(Debug, Clone)]
    pub struct Bus {
        pub bytes: Vec<u8>,
    }

    impl Bus {
        pub fn new(bytes: Vec<u8>) -> Self { Self { bytes } }
    }

    impl PartialEq for Bus {
        fn eq(&self, _other: &Self) -> bool { true }
    }

    impl super::Bus for Bus {
        fn read(&mut self, mut addr: u16) -> u8 {
            addr %= 0xc000; // Pretends the rom start is a 0x00
            self.bytes[addr as usize % self.bytes.len()]
        }

        fn write(&mut self, _addr: u16, _data: u8) {}
    }
}
