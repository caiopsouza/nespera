use crate::mos6502::bus::Bus;
use crate::mos6502::cycle;
use crate::mos6502::reg::Reg;

pub struct Opcode<'a> {
    pub reg: &'a mut Reg,
    pub bus: &'a mut Bus,
}

// First cycle is always fetching the opcode
impl<'a> Opcode<'a> {
    // Set the last cycle.
    // It's actually the next to last cycle but this is always incremented later.
    fn last_cycle(&mut self) { self.reg.set_next_to_last_cycle() }

    //region Codes

    pub fn nop(&mut self, _value: ()) {}
    pub fn dop(&mut self, _value: u8) {}

    pub fn lda(&mut self, value: u8) { self.reg.write_a(value) }
    pub fn ldx(&mut self, value: u8) { self.reg.write_x(value) }

    // endregion

    //region Read

    // Implied
    // 1    PC     R  fetch opcode, increment PC
    // 2    PC     R  read next instruction byte (and throw it away)
    pub fn implied(&mut self) -> Option<()> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.peek_pc(self.bus);
                self.last_cycle();
                Option::Some(())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Immediate
    // 1    PC     R  fetch opcode, increment PC
    // 2    PC     R  fetch value, increment PC
    pub fn immediate(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.last_cycle();
                Option::Some(self.reg.fetch_pc(self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Zero page
    // 1    PC     R  fetch opcode, increment PC
    // 2    PC     R  fetch address, increment PC
    // 3  address  R  read from effective address
    pub fn zero_page(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(self.bus);
                Option::None
            }
            cycle::T3 => {
                self.last_cycle();
                Option::Some(self.reg.peek_m(self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Zero page indexed
    // 1     PC      R  fetch opcode, increment PC
    // 2     PC      R  fetch address, increment PC
    // 3   address   R  read from address, add index register to it
    // 4  address+I* R  read from effective address
    fn zero_page_indexed(&mut self, index: u8) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m(self.bus);
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                self.last_cycle();
                Option::Some(self.reg.peek_m(self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn zero_page_x(&mut self) -> Option<u8> {
        let index = self.reg.get_x();
        self.zero_page_indexed(index)
    }

    pub fn zero_page_y(&mut self) -> Option<u8> {
        let index = self.reg.get_y();
        self.zero_page_indexed(index)
    }

    // Absolute
    // 1    PC     R  fetch opcode, increment PC
    // 2    PC     R  fetch low byte of address, increment PC
    // 3    PC     R  fetch high byte of address, increment PC
    // 4  address  R  read from effective address
    pub fn absolute(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.fetch_into_n(self.bus);
                Option::None
            }
            cycle::T4 => {
                self.last_cycle();
                Option::Some(self.reg.peek_absolute(self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Absolute indexed
    // 1     PC      fetch opcode, increment PC
    // 2     PC      fetch low byte of address, increment PC
    // 3     PC      fetch high byte of address,
    //               add index register to low address byte,
    //               increment PC
    // 4  address+* read from effective address,
    //               fix the high byte of effective address
    // 5+ address+I  re-read from effective address
    fn absolute_indexed(&mut self, index: u8) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.fetch_into_n(self.bus);
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                let internal_overflow = self.reg.get_internal_overflow();

                let value = self.reg.peek_absolute(self.bus);
                self.reg.set_fix_carry_n();

                if !internal_overflow { self.last_cycle() }

                Option::Some(value)
            }
            cycle::T5 => {
                self.last_cycle();
                Option::Some(self.reg.peek_absolute(self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn absolute_x(&mut self) -> Option<u8> {
        let index = self.reg.get_x();
        self.absolute_indexed(index)
    }

    pub fn absolute_y(&mut self) -> Option<u8> {
        let index = self.reg.get_y();
        self.absolute_indexed(index)
    }

    // Indexed indirect by X
    // 1      PC       fetch opcode, increment PC
    // 2      PC       fetch pointer address, increment PC
    // 3    pointer    read from the address, add X to it
    // 4   pointer+X   fetch effective address low
    // 5  pointer+X+1  fetch effective address high
    // 6    address    read from effective address
    pub fn indirect_x(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m(self.bus);
                self.reg.write_inc_m_by_x();
                let m = self.reg.read_m();
                self.reg.write_n(m.wrapping_add(1));
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_m_at_self(self.bus);
                Option::None
            }
            cycle::T5 => {
                self.reg.peek_n_at_self(self.bus);
                Option::None
            }
            cycle::T6 => {
                self.last_cycle();
                Option::Some(self.reg.peek_absolute(self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Indirect indexed by Y
    // 1      PC       R  fetch opcode, increment PC
    // 2      PC       R  fetch pointer address, increment PC
    // 3    pointer    R  fetch effective address low
    // 4   pointer+1   R  fetch effective address high,
    //                    add Y to low byte of effective address
    // 5   address+Y*  R  read from effective address,
    //                    fix high byte of effective address
    // 6+  address+Y   R  read from effective address
    pub fn indirect_y(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                let m = self.reg.fetch_into_m(self.bus);
                self.reg.write_n(m.wrapping_add(1));
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m_at_self(self.bus);
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_n_at_self(self.bus);
                self.reg.write_inc_m_by_y();
                Option::None
            }
            cycle::T5 => {
                let internal_overflow = self.reg.get_internal_overflow();

                let value = self.reg.peek_absolute(self.bus);
                self.reg.set_fix_carry_n();

                if !internal_overflow { self.last_cycle() }

                Option::Some(value)
            }
            cycle::T6 => {
                self.last_cycle();
                Option::Some(self.reg.peek_absolute(self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    //endregion
}
