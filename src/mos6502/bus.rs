#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Bus {
    pub bytes: Vec<u8>,
}

impl Bus {
    pub fn new(bytes: Vec<u8>) -> Self { Self { bytes } }

    // Map an address to some data for reading and writing.
    // Reading a Bus on the NES can have side effects, so mapping is always mutable.
    fn map(&mut self, mut addr: u16) -> &mut u8 {
        addr %= 0xc000; // Pretends the rom start is a 0x00.
        let addr = addr as usize % self.bytes.len();
        unsafe { self.bytes.get_unchecked_mut(addr) }
    }

    pub fn read(&mut self, addr: u16) -> u8 { *self.map(addr) }
    pub fn write(&mut self, addr: u16, data: u8) { *self.map(addr) = data }
}
