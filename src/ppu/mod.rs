use crate::bus::Bus;

pub struct Ppu<'a> {
    _bus: &'a mut Bus,
}

impl<'a> Ppu<'a> {
    pub fn new(_bus: &'a mut Bus) -> Ppu { Self { _bus } }
}