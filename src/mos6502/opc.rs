use crate::mos6502::cpu::Cpu;
use crate::mos6502::cycle;
use crate::mos6502::flags;

impl Cpu {
    // region Miscellaneous

    // Set current cycle to the last one.
    // It's actually the next to last cycle but this is always incremented later.
    // First cycle is always fetching the opcode
    fn finish(&mut self) { self.reg.set_next_to_last_cycle() }

    fn peek(&mut self, addr: u16) -> u8 { self.reg.peek_addr(&mut self.bus, addr) }
    fn poke(&mut self, addr: u16, data: u8) { self.reg.poke_addr(&mut self.bus, addr, data) }

    // endregion

    // region Codes

    // region Halt

    // Reading mode for the Kil opcode.
    // 2    PC     R  read next instruction byte (and throw it away). Revert cycle to the first one so it won't advance.
    pub fn kil(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.reg.peek_pc(&mut self.bus); }
            cycle::T3 => { self.reg.set_first_cycle(); }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // region Nop

    pub fn nop(&mut self, _value: ()) {}
    pub fn dop(&mut self, _value: u8) {}

    // endregion

    // region Load

    pub fn lda(&mut self, value: u8) { self.reg.write_a(value) }
    pub fn ldx(&mut self, value: u8) { self.reg.write_x(value) }
    pub fn ldy(&mut self, value: u8) { self.reg.write_y(value) }
    pub fn lax(&mut self, value: u8) {
        self.reg.write_a(value);
        self.reg.write_x(value);
    }

    // endregion

    // region Store
    pub fn sta(&mut self, addr: u16) {
        let data = self.reg.get_a();
        self.poke(addr, data)
    }
    pub fn stx(&mut self, addr: u16) {
        let data = self.reg.get_x();
        self.poke(addr, data)
    }
    pub fn sty(&mut self, addr: u16) {
        let data = self.reg.get_y();
        self.poke(addr, data)
    }

    // endregion

    // region Transfer

    pub fn tax(&mut self, _value: ()) {
        let value = self.reg.get_a();
        self.reg.write_x(value);
    }

    pub fn tay(&mut self, _value: ()) {
        let value = self.reg.get_a();
        self.reg.write_y(value);
    }

    pub fn txa(&mut self, _value: ()) {
        let value = self.reg.get_x();
        self.reg.write_a(value);
    }

    pub fn tya(&mut self, _value: ()) {
        let value = self.reg.get_y();
        self.reg.write_a(value);
    }

    // endregion

    // region Stack

    pub fn tsx(&mut self, _value: ()) {
        let value = self.reg.get_s();
        self.reg.write_x(value);
    }

    pub fn txs(&mut self, _value: ()) {
        let value = self.reg.get_x();
        self.reg.write_s(value);
    }

    pub fn pha(&mut self, addr: u16) {
        let data = self.reg.get_a();
        self.poke(addr, data);
    }

    pub fn php(&mut self, addr: u16) {
        let data = self.reg.get_p() | flags::BREAK_COMMAND | flags::UNUSED;
        self.poke(addr, data.into());
    }

    pub fn pla(&mut self, data: u8) {
        self.reg.write_a(data);
    }

    pub fn plp(&mut self, data: u8) {
        // Break Command and Unused are ignored when pulling
        let mut data = flags::Flags(data);
        data.copy(self.reg.get_p(), flags::BREAK_COMMAND | flags::UNUSED);
        self.reg.write_p(data);
    }

    // endregion

    // region Logic

    pub fn and(&mut self, data: u8) {
        let data = data & self.reg.get_a();
        self.reg.write_a(data);
    }

    pub fn eor(&mut self, data: u8) {
        let data = data ^ self.reg.get_a();
        self.reg.write_a(data);
    }

    pub fn ora(&mut self, data: u8) {
        let data = data | self.reg.get_a();
        self.reg.write_a(data);
    }

    pub fn bit(&mut self, data: u8) {
        self.reg.write_bit_test(data);
    }

    pub fn anc(&mut self, data: u8) {
        let data = data & self.reg.get_a();
        self.reg.write_a(data);

        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, p.get_negative())
    }

    pub fn sax(&mut self, addr: u16) {
        let data = self.reg.get_a() & self.reg.get_x();
        self.poke(addr, data);
    }

    // endregion

    // region Arithmetic
    pub fn adc(&mut self, value: u8) {
        let res = self.reg.get_a() as u16 + value as u16 + self.reg.get_p().get_carry() as u16;

        let a = self.reg.get_a();
        let p = self.reg.get_p_mut();

        p.change_zero_negative(res as u8);

        // When adding, carry happens if bit 8 is set
        p.change(flags::CARRY, (res & 0x0100u16) != 0);

        // Overflow happens when the sign of the addends is the same and differs from the sign of the sum
        p.change(
            flags::OVERFLOW,
            (!(a ^ value) & (a ^ res as u8) & 0x80) != 0,
        );

        // Save the result
        self.reg.write_a(res as u8);
    }

    pub fn sbc(&mut self, value: u8) {
        // Since you should subtract (1 - carry) inverting the value
        // has the same effect as a two's complement after the carry is added
        // Carry is inverted.
        self.adc(!value);
    }

    // endregion

    // region Comparison

    pub fn cmp(&mut self, data: u8) {
        let source = self.reg.get_a();
        self.reg.get_p_mut().change_cmp(source, data)
    }

    pub fn cpx(&mut self, data: u8) {
        let source = self.reg.get_x();
        self.reg.get_p_mut().change_cmp(source, data)
    }

    pub fn cpy(&mut self, data: u8) {
        let source = self.reg.get_y();
        self.reg.get_p_mut().change_cmp(source, data)
    }

    pub fn dcp(&mut self, (addr, data): (u16, u8)) {
        let source = self.reg.get_a();
        let data = data.wrapping_sub(1);
        self.poke(addr, data);
        self.reg.get_p_mut().change_cmp(source, data)
    }

    // endregion

    // region Increment

    pub fn inc(&mut self, (addr, data): (u16, u8)) {
        let data = data.wrapping_add(1);
        self.reg.get_p_mut().change_zero_negative(data);
        self.poke(addr, data)
    }

    pub fn inx(&mut self, _value: ()) {
        let data = self.reg.get_x().wrapping_add(1);
        self.reg.write_x(data)
    }

    pub fn iny(&mut self, _value: ()) {
        let data = self.reg.get_y().wrapping_add(1);
        self.reg.write_y(data)
    }

    pub fn isc(&mut self, (addr, data): (u16, u8)) {
        let data = data.wrapping_add(1);
        self.sbc(data);
        self.poke(addr, data);
    }

    // endregion

    // region Decrement

    pub fn dec(&mut self, (addr, data): (u16, u8)) {
        let data = data.wrapping_sub(1);
        self.reg.get_p_mut().change_zero_negative(data);
        self.poke(addr, data)
    }

    pub fn dex(&mut self, _value: ()) {
        let data = self.reg.get_x().wrapping_sub(1);
        self.reg.write_x(data)
    }

    pub fn dey(&mut self, _value: ()) {
        let data = self.reg.get_y().wrapping_sub(1);
        self.reg.write_y(data)
    }

    // endregion

    // region Shift

    fn shift(&mut self, addr: u16, res: u8, condition: bool) {
        let p = self.reg.get_p_mut();

        p.change(flags::CARRY, condition);
        p.change_zero_negative(res);

        self.poke(addr, res);
    }

    pub fn asl(&mut self, (addr, data): (u16, u8)) -> u8 {
        let res = data << 1;
        self.shift(addr,
                   res,
                   (data & 0b10000000) != 0);
        res
    }

    pub fn asl_acc(&mut self, data: u8) {
        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, (data & 0b10000000) != 0);
        self.reg.write_a(data << 1);
    }

    pub fn lsr(&mut self, (addr, data): (u16, u8)) -> u8 {
        let res = data >> 1;
        self.shift(addr,
                   res,
                   (data & 0b00000001) != 0);
        res
    }

    pub fn lsr_acc(&mut self, data: u8) {
        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, (data & 0b00000001) != 0);
        self.reg.write_a(data >> 1);
    }

    pub fn rol(&mut self, (addr, data): (u16, u8)) -> u8 {
        let carry = self.reg.get_p().get_carry();
        let res = (data << 1) | (carry as u8);
        self.shift(addr,
                   res,
                   (data & 0b10000000) != 0);
        res
    }

    pub fn rol_acc(&mut self, data: u8) {
        let p = self.reg.get_p_mut();
        let carry = p.get_carry();
        p.change(flags::CARRY, (data & 0b10000000) != 0);
        self.reg.write_a((data << 1) | (carry as u8));
    }

    pub fn ror(&mut self, (addr, data): (u16, u8)) -> u8 {
        let carry = self.reg.get_p().get_carry();
        let res = (data >> 1) | ((carry as u8) << 7);
        self.shift(addr,
                   res,
                   (data & 0b00000001) != 0);
        res
    }

    pub fn ror_acc(&mut self, data: u8) {
        let p = self.reg.get_p_mut();
        let carry = p.get_carry();
        p.change(flags::CARRY, (data & 0b00000001) != 0);
        self.reg.write_a((data >> 1) | ((carry as u8) << 7));
    }

    pub fn slo(&mut self, (addr, data): (u16, u8)) {
        let data = self.asl((addr, data));
        self.ora(data);
    }

    pub fn sre(&mut self, (addr, data): (u16, u8)) {
        let data = self.lsr((addr, data));
        self.eor(data);
    }

    pub fn rla(&mut self, (addr, data): (u16, u8)) {
        let data = self.rol((addr, data));
        self.and(data);
    }

    pub fn alr(&mut self, (addr, data): (u16, u8)) {
        let a = self.reg.get_a();
        self.shift(addr,
                   (data & a) >> 1,
                   (data & a & 0b00000001) != 0);
    }

    pub fn rra(&mut self, (addr, data): (u16, u8)) {
        let data = self.ror((addr, data));
        self.adc(data);
    }

    pub fn aar(&mut self, data: u8) {
        let carry = self.reg.get_p().get_carry();
        let a = self.reg.get_a();

        let value = data & a;
        let res = (value >> 1) | ((carry as u8) << 7);

        self.reg.get_p_mut().change(flags::CARRY, (data & 0b10000000) != 0);
        self.reg.write_a(res);
    }

    // endregion

    // region Jump

    // Jump to absolute address
    // 2    PC     R  fetch low address byte, increment PC
    // 3    PC     R  copy low address byte to PCL, fetch high address byte to PCH
    pub fn jmp_absolute(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.reg.fetch_into_m(&mut self.bus); }
            cycle::T3 => {
                self.reg.fetch_into_n(&mut self.bus);
                let pc = self.reg.get_absolute();
                self.reg.write_pc(pc);
                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Jump to indirect address
    // 2     PC      R  fetch pointer address low, increment PC
    // 3     PC      R  fetch pointer address high, increment PC
    // 4   pointer   R  fetch low address to latch
    // 5  pointer+1* R  fetch PCH, copy latch to PCL
    pub fn jmp_indirect(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.reg.fetch_into_m(&mut self.bus); }
            cycle::T3 => { self.reg.fetch_into_n(&mut self.bus); }
            cycle::T4 => { self.reg.peek_absolute_to_q(&mut self.bus); }
            cycle::T5 => {
                let pcl = self.reg.get_q();
                self.reg.write_inc_m(1);
                self.reg.peek_absolute_to_q(&mut self.bus);
                self.reg.write_pcl_pch(pcl, self.reg.get_q());
                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Jump to Subroutine
    // 2    PC     R  fetch low address byte, increment PC
    // 3  $0100,S  R  Internal operation
    // 4  $0100,S  W  push PCH on stack, decrement S
    // 5  $0100,S  W  push PCL on stack, decrement S
    // 6    PC     R  copy low address byte to PCL, fetch high address byte to PCH
    pub fn jsr(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.reg.fetch_into_m(&mut self.bus); }
            cycle::T3 => {}
            cycle::T4 => {
                self.reg.poke_at_stack(&mut self.bus, (self.reg.get_pc() >> 8) as u8);
                self.reg.set_inc_s(-1)
            }
            cycle::T5 => {
                self.reg.poke_at_stack(&mut self.bus, self.reg.get_pc() as u8);
                self.reg.set_inc_s(-1)
            }
            cycle::T6 => {
                self.reg.fetch_into_n(&mut self.bus);
                let m = self.reg.get_m();
                let n = self.reg.get_n();
                self.reg.write_pcl_pch(m, n);
                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Return from Interrupt
    // 2    PC     R  read next instruction byte (and throw it away)
    // 3  $0100,S  R  increment S
    // 4  $0100,S  R  pull P from stack, increment S
    // 5  $0100,S  R  pull PCL from stack, increment S
    // 6  $0100,S  R  pull PCH from stack
    pub fn rti(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.reg.prefetch_pc(&mut self.bus); }
            cycle::T3 => { self.reg.set_inc_s(1) }
            cycle::T4 => {
                let mut p: flags::Flags = (self.reg.peek_at_stack(&mut self.bus)).into();
                p.copy(self.reg.get_p(), flags::BREAK_COMMAND | flags::UNUSED);
                self.reg.write_p(p);
                self.reg.set_inc_s(1)
            }
            cycle::T5 => {
                let pcl = self.reg.peek_at_stack(&mut self.bus);
                self.reg.write_pcl(pcl);
                self.reg.set_inc_s(1)
            }
            cycle::T6 => {
                let pch = self.reg.peek_at_stack(&mut self.bus);
                self.reg.write_pch(pch);
                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Return from subroutine
    // 2    PC     R  read next instruction byte (and throw it away)
    // 3  $0100,S  R  increment S
    // 4  $0100,S  R  pull PCL from stack, increment S
    // 5  $0100,S  R  pull PCH from stack
    // 6    PC     R  increment PC
    pub fn rts(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.reg.prefetch_pc(&mut self.bus); }
            cycle::T3 => { self.reg.set_inc_s(1) }
            cycle::T4 => {
                let pcl = self.reg.peek_at_stack(&mut self.bus);
                self.reg.write_pcl(pcl);
                self.reg.set_inc_s(1);
            }
            cycle::T5 => {
                let pch = self.reg.peek_at_stack(&mut self.bus);
                self.reg.write_pch(pch);
            }
            cycle::T6 => {
                self.reg.set_next_pc();
                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // region Branch

    pub fn bcs(&mut self) { self.relative(self.reg.get_p().get_carry()) }
    pub fn bcc(&mut self) { self.relative(!self.reg.get_p().get_carry()) }
    pub fn beq(&mut self) { self.relative(self.reg.get_p().get_zero()) }
    pub fn bne(&mut self) { self.relative(!self.reg.get_p().get_zero()) }
    pub fn bmi(&mut self) { self.relative(self.reg.get_p().get_negative()) }
    pub fn bpl(&mut self) { self.relative(!self.reg.get_p().get_negative()) }
    pub fn bvs(&mut self) { self.relative(self.reg.get_p().get_overflow()) }
    pub fn bvc(&mut self) { self.relative(!self.reg.get_p().get_overflow()) }

    // endregion

    // region Status flags
    pub fn clc(&mut self, _value: ()) { self.reg.get_p_mut().clear(flags::CARRY) }
    pub fn cld(&mut self, _value: ()) { self.reg.get_p_mut().clear(flags::DECIMAL_MODE) }
    pub fn cli(&mut self, _value: ()) { self.reg.get_p_mut().clear(flags::INTERRUPT_DISABLE) }
    pub fn clv(&mut self, _value: ()) { self.reg.get_p_mut().clear(flags::OVERFLOW) }
    pub fn sec(&mut self, _value: ()) { self.reg.get_p_mut().set(flags::CARRY) }
    pub fn sed(&mut self, _value: ()) { self.reg.get_p_mut().set(flags::DECIMAL_MODE) }
    pub fn sei(&mut self, _value: ()) { self.reg.get_p_mut().set(flags::INTERRUPT_DISABLE) }

    // endregion

    // endregion

    // region Addressing Mode

    // region Write
    // 2    PC     R  read next instruction byte (and throw it away)
    // 3  $0100,S  W  push register on stack, decrement S
    pub fn w_stack(&mut self) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.peek_pc(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.set_inc_s(-1);
                self.finish();
                Option::Some(self.reg.get_stack_addr().wrapping_add(1))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Zero page
    // 2    PC     R  fetch address, increment PC
    // 3  address  R  read from effective address
    pub fn w_zero_page(&mut self) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.finish();
                Option::Some(self.reg.get_m() as u16)
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Zero page indexed
    // 2     PC      R  fetch address, increment PC
    // 3   address   R  read from address, add index register to it
    // 4  address+I* R  read from effective address
    fn w_zero_page_indexed(&mut self, index: u8) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m(&mut self.bus);
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                self.finish();
                Option::Some(self.reg.get_m() as u16)
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn w_zero_page_x(&mut self) -> Option<u16> {
        let index = self.reg.get_x();
        self.w_zero_page_indexed(index)
    }

    pub fn w_zero_page_y(&mut self) -> Option<u16> {
        let index = self.reg.get_y();
        self.w_zero_page_indexed(index)
    }

    // Absolute
    // 2    PC     fetch low byte of address, increment PC
    // 3    PC     fetch high byte of address, increment PC
    // 4  address  read from effective address
    pub fn w_absolute(&mut self) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.fetch_into_n(&mut self.bus);
                Option::None
            }
            cycle::T4 => {
                self.finish();
                Option::Some(self.reg.get_absolute())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Absolute indexed
    // 2     PC      fetch low byte of address, increment PC
    // 3     PC      fetch high byte of address, add index register to low address byte, increment PC
    // 4  address+I* read from effective address, fix the high byte of effective address
    // 5  address+I  write to effective address
    fn w_absolute_indexed(&mut self, index: u8) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.fetch_into_n(&mut self.bus);
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                self.reg.get_absolute();
                self.reg.set_fix_carry_n();
                Option::None
            }
            cycle::T5 => {
                self.finish();
                Option::Some(self.reg.get_absolute())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn w_absolute_x(&mut self) -> Option<u16> {
        let index = self.reg.get_x();
        self.w_absolute_indexed(index)
    }

    pub fn w_absolute_y(&mut self) -> Option<u16> {
        let index = self.reg.get_y();
        self.w_absolute_indexed(index)
    }

    // Indexed indirect by X
    // 2      PC       fetch pointer address, increment PC
    // 3    pointer    read from the address, add X to it
    // 4   pointer+X   fetch effective address low
    // 5  pointer+X+1  fetch effective address high
    // 6    address    read from effective address
    pub fn w_indirect_x(&mut self) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m(&mut self.bus);
                self.reg.write_inc_m_by_x();
                let m = self.reg.read_m();
                self.reg.write_n(m.wrapping_add(1));
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_m_to_self(&mut self.bus);
                Option::None
            }
            cycle::T5 => {
                self.reg.peek_n_to_self(&mut self.bus);
                Option::None
            }
            cycle::T6 => {
                self.finish();
                Option::Some(self.reg.get_absolute())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Indirect indexed by Y
    // 2      PC       fetch pointer address, increment PC
    // 3    pointer    fetch effective address low
    // 4   pointer+1   fetch effective address high, add Y to low byte of effective address
    // 5   address+Y*  read from effective address, fix high byte of effective address
    // 6   address+Y   write to effective address
    pub fn w_indirect_y(&mut self) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                let m = self.reg.fetch_into_m(&mut self.bus);
                self.reg.write_n(m.wrapping_add(1));
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m_to_self(&mut self.bus);
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_n_to_self(&mut self.bus);
                self.reg.write_inc_m_by_y();
                Option::None
            }
            cycle::T5 => {
                self.reg.get_absolute();
                self.reg.set_fix_carry_n();
                Option::None
            }
            cycle::T6 => {
                self.finish();
                Option::Some(self.reg.get_absolute())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // region Read

    // Implied
    // 2    PC     read next instruction byte (and throw it away)
    pub fn implied(&mut self) -> Option<()> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.peek_pc(&mut self.bus);
                self.finish();
                Option::Some(())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Accumulator
    // 2    PC     read next instruction byte (and throw it away)
    pub fn accumulator(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.peek_pc(&mut self.bus);
                self.finish();
                Option::Some(self.reg.read_a())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Immediate
    // 2    PC     fetch value, increment PC
    pub fn immediate(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.finish();
                Option::Some(self.reg.fetch_pc(&mut self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Stack
    // 2    PC     read next instruction byte (and throw it away)
    // 3  $0100,S  increment S
    // 4  $0100,S  pull register from stack
    pub fn r_stack(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.peek_pc(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.set_inc_s(1);
                Option::None
            }
            cycle::T4 => {
                self.finish();
                Option::Some(self.reg.peek_at_stack(&mut self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Zero Page
    pub fn r_zero_page(&mut self) -> Option<u8> {
        let addr = self.w_zero_page()?;
        Option::Some(self.reg.peek_at_m(&mut self.bus, addr))
    }

    pub fn r_zero_page_x(&mut self) -> Option<u8> {
        let index = self.reg.get_x();
        let addr = self.w_zero_page_indexed(index)?;
        Option::Some(self.reg.peek_at_m(&mut self.bus, addr))
    }

    pub fn r_zero_page_y(&mut self) -> Option<u8> {
        let index = self.reg.get_y();
        let addr = self.w_zero_page_indexed(index)?;
        Option::Some(self.reg.peek_at_m(&mut self.bus, addr))
    }

    // Absolute
    pub fn r_absolute(&mut self) -> Option<u8> {
        let addr = self.w_absolute()?;
        Option::Some(self.reg.peek_at_m(&mut self.bus, addr))
    }

    // Absolute indexed
    // 2     PC      fetch low byte of address, increment PC
    // 3     PC      fetch high byte of address, add index register to low address byte, increment PC
    // 4  address+*  read from effective address, fix the high byte of effective address
    // 5+ address+I  re-read from effective address
    fn r_absolute_indexed(&mut self, index: u8) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.fetch_into_n(&mut self.bus);
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                let value = self.reg.peek_absolute(&mut self.bus);
                self.reg.set_fix_carry_n();

                if !self.reg.get_internal_overflow() { self.finish() }

                Option::Some(value)
            }
            cycle::T5 => {
                self.finish();
                Option::Some(self.reg.peek_absolute(&mut self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn r_absolute_x(&mut self) -> Option<u8> {
        let index = self.reg.get_x();
        self.r_absolute_indexed(index)
    }

    pub fn r_absolute_y(&mut self) -> Option<u8> {
        let index = self.reg.get_y();
        self.r_absolute_indexed(index)
    }

    // Indexed indirect by X
    pub fn r_indirect_x(&mut self) -> Option<u8> {
        let addr = self.w_indirect_x()?;
        Option::Some(self.peek(addr))
    }

    // Indirect indexed by Y. Writing requires the 6th cycle.
    // 2      PC       fetch pointer address, increment PC
    // 3    pointer    fetch effective address low
    // 4   pointer+1   fetch effective address high, add Y to low byte of effective address
    // 5   address+Y*  read from effective address, fix high byte of effective address
    // 6+  address+Y   read from effective address
    pub fn r_indirect_y(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                let m = self.reg.fetch_into_m(&mut self.bus);
                self.reg.write_n(m.wrapping_add(1));
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m_to_self(&mut self.bus);
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_n_to_self(&mut self.bus);
                self.reg.write_inc_m_by_y();
                Option::None
            }
            cycle::T5 => {
                let value = self.reg.peek_absolute(&mut self.bus);
                self.reg.set_fix_carry_n();

                if !self.reg.get_internal_overflow() { self.finish() }

                Option::Some(value)
            }
            cycle::T6 => {
                self.finish();
                Option::Some(self.reg.peek_absolute(&mut self.bus))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Relative
    // 2     PC      fetch operand, increment PC. If branch not taken, finish.
    // 3+    PC      Prefetch opcode of next instruction. Add operand to PCL. If there was no overflow, finish.
    // 4+    PC*     Prefetch opcode of next instruction. Fix PCH.
    pub fn relative(&mut self, branch_taken: bool) {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                if !branch_taken { self.finish() }
            }
            cycle::T3 => {
                self.reg.prefetch_pc(&mut self.bus);

                let m = self.reg.get_m();
                self.reg.write_inc_pcl(m as i8);

                if !self.reg.get_internal_overflow() { self.finish() }
            }
            cycle::T4 => {
                self.reg.prefetch_pc(&mut self.bus);
                self.reg.set_fix_carry_pc();
                self.finish()
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // region Read / Write

    // Zero page
    // 2    PC     fetch address, increment PC
    // 3  address  read from effective address
    // 4  address  write the value back to effective address, and do the operation on it
    // 5  address  write the new value to effective address
    pub fn rw_zero_page(&mut self) -> Option<(u16, u8)> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m_to_n(&mut self.bus);
                Option::None
            }
            cycle::T4 => {
                self.reg.poke_n_to_m(&mut self.bus);
                Option::None
            }
            cycle::T5 => {
                self.finish();
                Option::Some((self.reg.get_m() as u16, self.reg.get_n()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Zero page indexed
    // 2     PC      fetch address, increment PC
    // 3   address   read from address, add index register X to it
    // 4  address+X* read from effective address
    // 5  address+X* write the value back to effective address, and do the operation on it
    // 6  address+X* write the new value to effective address
    fn rw_zero_page_indexed(&mut self, index: u8) -> Option<(u16, u8)> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m(&mut self.bus);
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_m_to_n(&mut self.bus);
                Option::None
            }
            cycle::T5 => {
                self.reg.poke_n_to_m(&mut self.bus);
                Option::None
            }
            cycle::T6 => {
                self.finish();
                Option::Some((self.reg.get_m() as u16, self.reg.get_n()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn rw_zero_page_x(&mut self) -> Option<(u16, u8)> {
        let index = self.reg.get_x();
        self.rw_zero_page_indexed(index)
    }

    pub fn rw_zero_page_y(&mut self) -> Option<(u16, u8)> {
        let index = self.reg.get_y();
        self.rw_zero_page_indexed(index)
    }

    // Absolute
    // 2    PC     fetch low byte of address, increment PC
    // 3    PC     fetch high byte of address, increment PC
    // 4  address  read from effective address
    // 5  address  write the value back to effective address, and do the operation on it
    // 6  address  write the new value to effective address
    pub fn rw_absolute(&mut self) -> Option<(u16, u8)> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.fetch_into_n(&mut self.bus);
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_absolute_to_q(&mut self.bus);
                Option::None
            }
            cycle::T5 => {
                self.reg.poke_q_to_absolute(&mut self.bus);
                Option::None
            }
            cycle::T6 => {
                self.finish();
                Option::Some((self.reg.get_absolute(), self.reg.get_q()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Absolute indexed
    // 2    PC       fetch low byte of address, increment PC
    // 3    PC       fetch high byte of address, add index register to low address byte, increment PC
    // 4  address+I* read from effective address, fix the high byte of effective address
    // 5  address+I  re-read from effective address
    // 6  address+I  write the value back to effective address, and do the operation on it
    // 7  address+I  write the new value to effective address
    fn rw_absolute_indexed(&mut self, index: u8) -> Option<(u16, u8)> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.fetch_into_n(&mut self.bus);
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_absolute(&mut self.bus);
                self.reg.set_fix_carry_n();
                Option::None
            }
            cycle::T5 => {
                self.reg.peek_absolute_to_q(&mut self.bus);
                Option::None
            }
            cycle::T6 => {
                self.reg.poke_q_to_absolute(&mut self.bus);
                Option::None
            }
            cycle::T7 => {
                self.finish();
                Option::Some((self.reg.get_absolute(), self.reg.get_q()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn rw_absolute_x(&mut self) -> Option<(u16, u8)> {
        let index = self.reg.get_x();
        self.rw_absolute_indexed(index)
    }

    pub fn rw_absolute_y(&mut self) -> Option<(u16, u8)> {
        let index = self.reg.get_y();
        self.rw_absolute_indexed(index)
    }

    // Indexed indirect by X
    // 2      PC       fetch pointer address, increment PC
    // 3    pointer    read from the address, add X to it
    // 4   pointer+X   fetch effective address low
    // 5  pointer+X+1  fetch effective address high
    // 6    address    read from effective address
    // 7    address    write the value back to effective address, and do the operation on it
    // 8    address    write the new value to effective address
    pub fn rw_indirect_x(&mut self) -> Option<(u16, u8)> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.reg.fetch_into_m(&mut self.bus);
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m(&mut self.bus);
                self.reg.write_inc_m_by_x();
                let m = self.reg.read_m();
                self.reg.write_n(m.wrapping_add(1));
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_m_to_self(&mut self.bus);
                Option::None
            }
            cycle::T5 => {
                self.reg.peek_n_to_self(&mut self.bus);
                Option::None
            }
            cycle::T6 => {
                self.reg.peek_absolute_to_q(&mut self.bus);
                Option::None
            }
            cycle::T7 => {
                self.reg.poke_q_to_absolute(&mut self.bus);
                Option::None
            }
            cycle::T8 => {
                self.finish();
                Option::Some((self.reg.get_absolute(), self.reg.get_q()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Indirect indexed by Y
    // 2      PC       fetch pointer address, increment PC
    // 3    pointer    fetch effective address low
    // 4   pointer+1   fetch effective address high, add Y to low byte of effective address
    // 5   address+Y*  read from effective address, fix high byte of effective address
    // 6   address+Y   read from effective address
    // 7   address+Y   write the value back to effective address, and do the operation on it
    // 8   address+Y   write the new value to effective address
    pub fn rw_indirect_y(&mut self) -> Option<(u16, u8)> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                let m = self.reg.fetch_into_m(&mut self.bus);
                self.reg.write_n(m.wrapping_add(1));
                Option::None
            }
            cycle::T3 => {
                self.reg.peek_m_to_self(&mut self.bus);
                Option::None
            }
            cycle::T4 => {
                self.reg.peek_n_to_self(&mut self.bus);
                self.reg.write_inc_m_by_y();
                Option::None
            }
            cycle::T5 => {
                self.reg.peek_absolute(&mut self.bus);
                self.reg.set_fix_carry_n();

                Option::None
            }
            cycle::T6 => {
                self.reg.peek_absolute_to_q(&mut self.bus);
                Option::None
            }
            cycle::T7 => {
                self.reg.poke_q_to_absolute(&mut self.bus);
                Option::None
            }
            cycle::T8 => {
                self.finish();
                Option::Some((self.reg.get_absolute(), self.reg.get_q()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // endregion
}