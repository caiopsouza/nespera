use crate::cpu::Cpu;
use crate::cpu::cycle;
use crate::cpu::flags;
use crate::cpu::reg;
use crate::utils::bits;

impl Cpu {
    // region Miscellaneous

    // Set current cycle to the last one.
    // It's actually the next to last cycle but this is always incremented later.
    // First cycle is always fetching the opcode
    fn finish(&mut self) { self.reg.set_next_to_last_cycle() }

    fn read(&mut self, addr: u16) -> u8 {
        self.reg.addr_bus(addr, self.bus.borrow_mut().read(addr))
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.reg.addr_bus(addr, data);
        self.bus.borrow_mut().write(addr, data);
    }

    fn push(&mut self, data: u8) {
        let addr = self.reg.get_stack_addr();
        self.write(addr, data);
        self.reg.set_inc_s(-1);
    }

    fn pull(&mut self) -> u8 {
        let res = self.read_stack();
        self.reg.set_inc_s(1); // Should be pre decremented, so I do it later here.
        res
    }

    // Similar to push, but the write gate is not enabled
    fn push_write_disabled(&mut self, _data: u8) {
        self.reg.set_inc_s(-1);
    }

    // Internal registers
    fn fetch_into_m(&mut self) {
        let data = self.read_pc();
        self.reg.set_m(data);
        self.reg.set_internal_overflow(reg::InternalOverflow::None);
        self.reg.set_next_pc()
    }

    fn fetch_into_n(&mut self) {
        let n = self.read_pc();
        self.reg.set_n(n);
        self.reg.set_next_pc()
    }

    // PC
    fn read_pc(&mut self) -> u8 { self.read(self.reg.get_pc()) }
    fn prefetch_pc(&mut self) -> u8 { self.read(self.reg.get_pc()) }
    pub fn fetch_pc(&mut self) -> u8 {
        let res = self.prefetch_pc();
        self.reg.set_next_pc();
        res
    }

    // Read from register as an address to the external bus
    fn read_m(&mut self) -> u8 {
        self.read(self.reg.get_m() as u16)
    }

    fn read_at_m(&mut self, addr: u16) -> u8 {
        let data = self.read(addr);
        self.reg.write_m(data);
        data
    }

    fn read_m_to_self(&mut self) {
        let data = self.read(self.reg.get_m() as u16);
        self.reg.write_m(data)
    }

    fn read_n_to_self(&mut self) {
        let data = self.read(self.reg.get_n() as u16);
        self.reg.write_n(data)
    }

    fn read_m_to_n(&mut self) {
        let data = self.read(self.reg.get_m() as u16);
        self.reg.write_n(data)
    }

    fn read_absolute(&mut self) -> u8 {
        self.read(self.reg.get_absolute())
    }

    fn read_absolute_to_q(&mut self) {
        let data = self.read_absolute();
        self.reg.write_q(data)
    }

    // Write from register as an address to the external bus
    fn write_n_to_m(&mut self) {
        self.write(self.reg.get_m() as u16, self.reg.get_n())
    }

    fn write_q_to_absolute(&mut self) {
        self.write(self.reg.get_absolute(), self.reg.get_q())
    }

    // Pull a value from the stack
    fn read_stack(&mut self) -> u8 {
        let addr = self.reg.get_stack_addr();
        self.read(addr)
    }

    // endregion

    // region Codes

    // region Interrupts

    // Kil opcode.
    // 2    PC     read next instruction byte (and throw it away).
    // 3    PC     revert cycle to the first one so it won't advance.
    pub fn kil(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => {
                trace!(target: "opcode", "kil");
                self.read_pc();
            }
            cycle::T3 => {
                self.reg.set_first_cycle();
                panic!("Kil opcode finished running. Aborting program.");
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Break execution and load the interrupt vector
    // 2      PC      read next instruction byte (and throw it away), increment PC
    // 3    $0100,S   push PCH on stack (with B flag set), decrement S
    // 4    $0100,S   push PCL on stack, decrement S
    // 5    $0100,S   push P on stack, decrement S
    // 6     $FFXX    fetch PCL
    // 7   $FF(XX+1)  fetch PCH
    pub fn brk(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => {
                trace!(target: "opcode", "brk");
                self.fetch_pc();
            }
            cycle::T3 => { self.push(self.reg.get_pch()); }
            cycle::T4 => { self.push(self.reg.get_pcl()); }
            cycle::T5 => {
                // Choose the interrupt vector.
                let vector = if self.bus.borrow().nmi { 0xfa } else { 0xfe };
                self.reg.set_m(vector);
                self.reg.set_internal_overflow(reg::InternalOverflow::None);

                let mut p = self.reg.get_p() | flags::BREAK_COMMAND | flags::UNUSED;
                // IRQ or NMI clear the B flag
                {
                    let bus = self.bus.borrow();
                    if bus.irq || bus.nmi { p.clear(flags::BREAK_COMMAND) }
                }

                self.push(p.into())
            }
            cycle::T6 => {
                // NMI won't affect the I flag.
                {
                    let bus = self.bus.borrow();
                    if !bus.nmi { self.reg.get_p_mut().set(flags::INTERRUPT_DISABLE); }
                }

                let vector = self.reg.get_m();
                let pcl = self.read(0xff00 | vector as u16);
                self.reg.write_pcl(pcl);
            }
            cycle::T7 => {
                let vector = self.reg.get_m() + 1;
                let pch = self.read(0xff00 | vector as u16);
                self.reg.write_pch(pch);

                // Clear the interrupt flags
                {
                    let mut bus = self.bus.borrow_mut();
                    bus.nmi = false;
                    bus.irq = false;
                }

                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Reset subroutine. Load the reset vector.
    // Reset is a read cycle, so nothing is written to the bus even if it were supposed to.
    // 2    PC     read next instruction byte (and throw it away), increment PC
    // 3  $0100,S  Write disabled: should push PCH on stack (with B flag set), decrement S
    // 4  $0100,S  Write disabled: should push PCL on stack, decrement S
    // 5  $0100,S  Write disabled: should push P on stack, decrement S
    // 6   $FFFE   fetch PCL
    // 7   $FFFF   fetch PCH
    pub fn rst(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => {
                trace!(target: "opcode", "rst");
                self.fetch_pc();
            }
            cycle::T3 => self.push_write_disabled(self.reg.get_pch()),
            cycle::T4 => self.push_write_disabled(self.reg.get_pcl()),
            cycle::T5 => self.push_write_disabled(0),
            cycle::T6 => {
                let pcl = self.read(0xfffc);
                self.reg.write_pcl(pcl);
            }
            cycle::T7 => {
                let pcl = self.read(0xfffd);
                self.reg.write_pch(pcl);
                self.resetting = false;
                self.bus.borrow_mut().reset = false;
                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // region Nop

    pub fn nop(&mut self, _value: ()) {
        trace!(target: "opcode", "nop");
    }
    pub fn dop(&mut self, value: u8) {
        trace!(target: "opcode", "dop, data: 0x{:02x}", value);
    }

    // endregion

    // region Load

    pub fn lda(&mut self, value: u8) {
        trace!(target: "opcode", "lda, data: 0x{:02x}", value);
        self.reg.write_a(value)
    }
    pub fn ldx(&mut self, value: u8) {
        trace!(target: "opcode", "ldx, data: 0x{:02x}", value);
        self.reg.write_x(value)
    }
    pub fn ldy(&mut self, value: u8) {
        trace!(target: "opcode", "ldy, data: 0x{:02x}", value);
        self.reg.write_y(value)
    }
    pub fn lax(&mut self, value: u8) {
        trace!(target: "opcode", "lax, data: 0x{:02x}", value);
        self.reg.write_a(value);
        self.reg.write_x(value);
    }

    // endregion

    // region Store
    pub fn sta(&mut self, addr: u16) {
        trace!(target: "opcode", "sta, addr: 0x{:04x}", addr);
        let data = self.reg.get_a();
        self.write(addr, data)
    }
    pub fn stx(&mut self, addr: u16) {
        trace!(target: "opcode", "stx, addr: 0x{:04x}", addr);
        let data = self.reg.get_x();
        self.write(addr, data)
    }
    pub fn sty(&mut self, addr: u16) {
        trace!(target: "opcode", "sty, addr: 0x{:04x}", addr);
        let data = self.reg.get_y();
        self.write(addr, data)
    }

    // endregion

    // region Transfer

    pub fn tax(&mut self, _value: ()) {
        trace!(target: "opcode", "tax");
        let value = self.reg.get_a();
        self.reg.write_x(value);
    }

    pub fn tay(&mut self, _value: ()) {
        trace!(target: "opcode", "tay");
        let value = self.reg.get_a();
        self.reg.write_y(value);
    }

    pub fn txa(&mut self, _value: ()) {
        trace!(target: "opcode", "txa");
        let value = self.reg.get_x();
        self.reg.write_a(value);
    }

    pub fn tya(&mut self, _value: ()) {
        trace!(target: "opcode", "tya");
        let value = self.reg.get_y();
        self.reg.write_a(value);
    }

    pub fn oam(&mut self) {
        let cycle = self.reg.get_cycle();
        let index = (cycle / 2) as u8;

        if cycle == 1 { trace!(target: "opcode", "oam"); }

        if cycle % 2 == 1 {
            // Read cycle
            let mut bus = self.bus.borrow_mut();
            let addr = bits::word(bus.ppu.oam_source, index);
            let data = bus.read(addr);
            self.reg.set_m(data);
        } else {
            // Write cycle
            {
                let mut bus = self.bus.borrow_mut();
                let addr = bits::word(bus.ppu.oam_dest, index);
                let data = self.reg.get_m();
                bus.write(addr, data);
            }

            if index == 255 {
                self.oam_transferring = false;
                self.bus.borrow_mut().ppu.oam_transfer = false;
                self.finish();
            }
        }
    }

    // endregion

    // region Stack

    pub fn tsx(&mut self, _value: ()) {
        trace!(target: "opcode", "tsx");
        let value = self.reg.get_s();
        self.reg.write_x(value);
    }

    pub fn txs(&mut self, _value: ()) {
        trace!(target: "opcode", "txs");
        let value = self.reg.get_x();
        self.reg.write_s(value);
    }

    pub fn pha(&mut self, addr: u16) {
        let data = self.reg.get_a();
        trace!(target: "opcode", "pha, addr: 0x{:04x}, data: 0x{:02x}", addr, data);
        self.write(addr, data);
    }

    pub fn php(&mut self, addr: u16) {
        // Break Command and Unused are always set when pushing
        let data = self.reg.get_p() | flags::BREAK_COMMAND | flags::UNUSED;
        let data: u8 = data.into();
        trace!(target: "opcode", "php, addr: 0x{:04x}, data: 0x{:02x}", addr, data);
        self.write(addr, data);
    }

    pub fn pla(&mut self, data: u8) {
        trace!(target: "opcode", "pla, data: 0x{:02x}", data);
        self.reg.write_a(data);
    }

    pub fn plp(&mut self, data: u8) {
        trace!(target: "opcode", "plp, data: 0x{:02x}", data);
        // Break Command and Unused are ignored when pulling
        let mut data = flags::Flags(data);
        data.copy(self.reg.get_p(), flags::BREAK_COMMAND | flags::UNUSED);
        self.reg.write_p(data);
    }

    // endregion

    // region Logic

    pub fn and(&mut self, data: u8) {
        trace!(target: "opcode", "and, data: 0x{:02x}", data);
        let data = data & self.reg.get_a();
        self.reg.write_a(data);
    }

    pub fn eor(&mut self, data: u8) {
        trace!(target: "opcode", "eor, data: 0x{:02x}", data);
        let data = data ^ self.reg.get_a();
        self.reg.write_a(data);
    }

    pub fn ora(&mut self, data: u8) {
        trace!(target: "opcode", "ora, data: 0x{:02x}", data);
        let data = data | self.reg.get_a();
        self.reg.write_a(data);
    }

    pub fn bit(&mut self, data: u8) {
        trace!(target: "opcode", "bit, data: 0x{:02x}, a: 0x{:02x}", data, self.reg.get_a());
        self.reg.write_bit_test(data);
    }

    pub fn anc(&mut self, data: u8) {
        trace!(target: "opcode", "anc, data: 0x{:02x}", data);
        let data = data & self.reg.get_a();
        self.reg.write_a(data);

        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, p.get_negative())
    }

    pub fn xaa(&mut self, data: u8) {
        trace!(target: "opcode", "xaa, data: 0x{:02x}", data);
        let a = self.reg.get_a();

        // XAA has analogic behaviour.
        // "Magic" defines which bits of A will be used in the result and can vary wildly.
        // Below values happen over 98% of the time.
        let magic = if (a & 0x01) != 0 { 0xff } else { 0xfe };

        let data = (a | magic) & self.reg.get_x() & data;
        self.reg.write_a(data);
    }

    pub fn sax(&mut self, addr: u16) {
        trace!(target: "opcode", "sax, addr: 0x{:04x}", addr);
        let data = self.reg.get_a() & self.reg.get_x();
        self.write(addr, data);
    }

    // endregion

    // region Arithmetic

    pub fn adc(&mut self, data: u8) {
        trace!(target: "opcode", "adc, data: 0x{:02x}", data);
        let a = self.reg.get_a();
        let p = self.reg.get_p_mut();

        let res = a as u16 + data as u16 + p.get_carry() as u16;

        p.change_zero_negative(res as u8);

        // When adding, carry happens if bit 8 is set
        p.change(flags::CARRY, (res & 0x0100u16) != 0);

        // Overflow happens when the sign of the addends is the same and differs from the sign of the sum
        p.change(
            flags::OVERFLOW,
            (!(a ^ data) & (a ^ res as u8) & 0x80) != 0,
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
        trace!(target: "opcode", "cmp, data: 0x{:02x}", data);
        let source = self.reg.get_a();
        self.reg.get_p_mut().change_cmp(source, data)
    }

    pub fn cpx(&mut self, data: u8) {
        trace!(target: "opcode", "cpx, data: 0x{:02x}", data);
        let source = self.reg.get_x();
        self.reg.get_p_mut().change_cmp(source, data)
    }

    pub fn cpy(&mut self, data: u8) {
        trace!(target: "opcode", "cpy, data: 0x{:02x}", data);
        let source = self.reg.get_y();
        self.reg.get_p_mut().change_cmp(source, data)
    }

    pub fn dcp(&mut self, (addr, data): (u16, u8)) {
        trace!(target: "opcode", "dcp, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let source = self.reg.get_a();
        let data = data.wrapping_sub(1);
        self.write(addr, data);
        self.reg.get_p_mut().change_cmp(source, data)
    }

    pub fn axs(&mut self, data: u8) {
        trace!(target: "opcode", "axs, data: 0x{:02x}", data);
        let v = self.reg.get_p().get_overflow();

        let x = self.reg.get_a() & self.reg.get_x();
        self.reg.write_x(x.wrapping_sub(data));

        let p = self.reg.get_p_mut();
        p.change_cmp(x, data);

        // Overflow flag in not affected, so change to the old value.
        p.change(flags::OVERFLOW, v);
    }

    // endregion

    // region Increment

    pub fn inc(&mut self, (addr, data): (u16, u8)) {
        trace!(target: "opcode", "inc, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let data = data.wrapping_add(1);
        self.reg.get_p_mut().change_zero_negative(data);
        self.write(addr, data)
    }

    pub fn inx(&mut self, _value: ()) {
        trace!(target: "opcode", "inx");
        let data = self.reg.get_x().wrapping_add(1);
        self.reg.write_x(data)
    }

    pub fn iny(&mut self, _value: ()) {
        trace!(target: "opcode", "iny");
        let data = self.reg.get_y().wrapping_add(1);
        self.reg.write_y(data)
    }

    pub fn isc(&mut self, (addr, data): (u16, u8)) {
        trace!(target: "opcode", "isc, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let data = data.wrapping_add(1);
        self.sbc(data);
        self.write(addr, data);
    }

    // endregion

    // region Decrement

    pub fn dec(&mut self, (addr, data): (u16, u8)) {
        trace!(target: "opcode", "dec, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let data = data.wrapping_sub(1);
        self.reg.get_p_mut().change_zero_negative(data);
        self.write(addr, data)
    }

    pub fn dex(&mut self, _value: ()) {
        trace!(target: "opcode", "dex");
        let data = self.reg.get_x().wrapping_sub(1);
        self.reg.write_x(data)
    }

    pub fn dey(&mut self, _value: ()) {
        trace!(target: "opcode", "dey");
        let data = self.reg.get_y().wrapping_sub(1);
        self.reg.write_y(data)
    }

    // endregion

    // region Shift

    fn shift(&mut self, addr: u16, res: u8, condition: bool) {
        let p = self.reg.get_p_mut();

        p.change(flags::CARRY, condition);
        p.change_zero_negative(res);

        self.write(addr, res);
    }

    pub fn asl(&mut self, (addr, data): (u16, u8)) -> u8 {
        trace!(target: "opcode", "asl, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let res = data << 1;
        self.shift(addr,
                   res,
                   (data & 0b10000000) != 0);
        res
    }

    pub fn asl_acc(&mut self, data: u8) {
        trace!(target: "opcode", "asl, data: 0x{:02x}", data);
        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, (data & 0b10000000) != 0);
        self.reg.write_a(data << 1);
    }

    pub fn lsr(&mut self, (addr, data): (u16, u8)) -> u8 {
        trace!(target: "opcode", "lsr, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let res = data >> 1;
        self.shift(addr,
                   res,
                   (data & 0b00000001) != 0);
        res
    }

    pub fn lsr_acc(&mut self, data: u8) {
        trace!(target: "opcode", "lsr, data: 0x{:02x}", data);
        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, (data & 0b00000001) != 0);
        self.reg.write_a(data >> 1);
    }

    pub fn rol(&mut self, (addr, data): (u16, u8)) -> u8 {
        trace!(target: "opcode", "rol, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let carry = self.reg.get_p().get_carry();
        let res = (data << 1) | (carry as u8);
        self.shift(addr,
                   res,
                   (data & 0b10000000) != 0);
        res
    }

    pub fn rol_acc(&mut self, data: u8) {
        trace!(target: "opcode", "rol, data: 0x{:02x}", data);
        let p = self.reg.get_p_mut();
        let carry = p.get_carry();
        p.change(flags::CARRY, (data & 0b10000000) != 0);
        self.reg.write_a((data << 1) | (carry as u8));
    }

    pub fn ror(&mut self, (addr, data): (u16, u8)) -> u8 {
        trace!(target: "opcode", "ror, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let carry = self.reg.get_p().get_carry();
        let res = (data >> 1) | ((carry as u8) << 7);
        self.shift(addr,
                   res,
                   (data & 0b00000001) != 0);
        res
    }

    pub fn ror_acc(&mut self, data: u8) {
        trace!(target: "opcode", "ror, data: 0x{:02x}", data);
        let p = self.reg.get_p_mut();
        let carry = p.get_carry();
        p.change(flags::CARRY, (data & 0b00000001) != 0);
        self.reg.write_a((data >> 1) | ((carry as u8) << 7));
    }

    pub fn slo(&mut self, (addr, data): (u16, u8)) {
        trace!(target: "opcode", "slo, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let data = self.asl((addr, data));
        self.ora(data);
    }

    pub fn sre(&mut self, (addr, data): (u16, u8)) {
        trace!(target: "opcode", "sre, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let data = self.lsr((addr, data));
        self.eor(data);
    }

    pub fn rla(&mut self, (addr, data): (u16, u8)) {
        trace!(target: "opcode", "rla, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let data = self.rol((addr, data));
        self.and(data);
    }

    pub fn alr(&mut self, data: u8) {
        trace!(target: "opcode", "alr, data: 0x{:02x}", data);
        self.and(data);
        let data = self.reg.get_a();
        self.lsr_acc(data);
    }

    pub fn rra(&mut self, (addr, data): (u16, u8)) {
        trace!(target: "opcode", "rra, addr: 0x{:02x}, data: 0x{:02x}", addr, data);
        let data = self.ror((addr, data));
        self.adc(data);
    }

    pub fn arr(&mut self, data: u8) {
        trace!(target: "opcode", "arr, data: 0x{:02x}", data);
        let a = self.reg.get_a();
        let p = self.reg.get_p_mut();
        let carry = p.get_carry();

        let value = (data & a) >> 1;
        let res = value | ((carry as u8) << 7);

        p.toggle(flags::NEGATIVE);
        p.clear(flags::OVERFLOW | flags::CARRY);

        match value & 0x60 {
            0x20 => p.set(flags::OVERFLOW),
            0x40 => p.set(flags::OVERFLOW | flags::CARRY),
            0x60 => p.set(flags::CARRY),
            _ => {}
        }

        self.reg.write_a(res);
    }

    // endregion

    // region Jump

    // Jump to absolute address
    // 2    PC     fetch low address byte, increment PC
    // 3    PC     copy low address byte to PCL, fetch high address byte to PCH
    pub fn jmp_absolute(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.fetch_into_m(); }
            cycle::T3 => {
                self.fetch_into_n();
                let pc = self.reg.get_absolute();
                self.reg.write_pc(pc);
                trace!(target: "opcode", "jmp, addr: 0x{:04x}", self.reg.get_pc());
                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Jump to indirect address
    // 2     PC      fetch pointer address low, increment PC
    // 3     PC      fetch pointer address high, increment PC
    // 4   pointer   fetch low address to latch
    // 5  pointer+1* fetch PCH, copy latch to PCL
    pub fn jmp_indirect(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.fetch_into_m(); }
            cycle::T3 => { self.fetch_into_n(); }
            cycle::T4 => { self.read_absolute_to_q(); }
            cycle::T5 => {
                let pcl = self.reg.get_q();
                self.reg.write_inc_m(1);
                self.read_absolute_to_q();
                self.reg.write_pcl_pch(pcl, self.reg.get_q());
                trace!(target: "opcode", "jmp, addr: 0x{:04x}", self.reg.get_pc());
                self.finish();
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Jump to Subroutine
    // 2    PC     fetch low address byte, increment PC
    // 3  $0100,S  Internal operation
    // 4  $0100,S  push PCH on stack, decrement S
    // 5  $0100,S  push PCL on stack, decrement S
    // 6    PC     copy low address byte to PCL, fetch high address byte to PCH
    pub fn jsr(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.fetch_into_m(); }
            cycle::T3 => {}
            cycle::T4 => { self.push((self.reg.get_pc() >> 8) as u8) }
            cycle::T5 => { self.push(self.reg.get_pc() as u8) }
            cycle::T6 => {
                self.fetch_into_n();
                let m = self.reg.get_m();
                let n = self.reg.get_n();
                self.reg.write_pcl_pch(m, n);
                self.finish();
                trace!(target: "opcode", "jsr, addr: 0x{:04x}", self.reg.get_pc());
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Return from Interrupt
    // 2    PC     read next instruction byte (and throw it away)
    // 3  $0100,S  increment S
    // 4  $0100,S  pull P from stack, increment S
    // 5  $0100,S  pull PCL from stack, increment S
    // 6  $0100,S  pull PCH from stack
    pub fn rti(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.prefetch_pc(); }
            cycle::T3 => { self.reg.set_inc_s(1) }
            cycle::T4 => {
                let mut p: flags::Flags = (self.pull()).into();
                p.copy(self.reg.get_p(), flags::BREAK_COMMAND | flags::UNUSED);
                self.reg.write_p(p);
            }
            cycle::T5 => {
                let pcl = self.pull();
                self.reg.write_pcl(pcl);
            }
            cycle::T6 => {
                let pch = self.read_stack();
                self.reg.write_pch(pch);
                self.finish();
                trace!(target: "opcode", "rti, addr: 0x{:04x}", self.reg.get_pc());
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Return from subroutine
    // 2    PC     read next instruction byte (and throw it away)
    // 3  $0100,S  increment S
    // 4  $0100,S  pull PCL from stack, increment S
    // 5  $0100,S  pull PCH from stack
    // 6    PC     increment PC
    pub fn rts(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => { self.prefetch_pc(); }
            cycle::T3 => { self.reg.set_inc_s(1) }
            cycle::T4 => {
                let pcl = self.pull();
                self.reg.write_pcl(pcl);
            }
            cycle::T5 => {
                let pch = self.read_stack();
                self.reg.write_pch(pch);
            }
            cycle::T6 => {
                self.reg.set_next_pc();
                self.finish();
                trace!(target: "opcode", "rts, addr: 0x{:04x}", self.reg.get_pc());
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // region Branch

    pub fn bcs(&mut self) { self.relative(self.reg.get_p().get_carry(), "bcs") }
    pub fn bcc(&mut self) { self.relative(!self.reg.get_p().get_carry(), "bcc") }
    pub fn beq(&mut self) { self.relative(self.reg.get_p().get_zero(), "beq") }
    pub fn bne(&mut self) { self.relative(!self.reg.get_p().get_zero(), "bne") }
    pub fn bmi(&mut self) { self.relative(self.reg.get_p().get_negative(), "bmi") }
    pub fn bpl(&mut self) { self.relative(!self.reg.get_p().get_negative(), "bpl") }
    pub fn bvs(&mut self) { self.relative(self.reg.get_p().get_overflow(), "bvs") }
    pub fn bvc(&mut self) { self.relative(!self.reg.get_p().get_overflow(), "bvc") }

    // endregion

    // region Status flags

    pub fn clc(&mut self, _value: ()) {
        trace!(target: "opcode", "clc");
        self.reg.get_p_mut().clear(flags::CARRY)
    }
    pub fn cld(&mut self, _value: ()) {
        trace!(target: "opcode", "cld");
        self.reg.get_p_mut().clear(flags::DECIMAL_MODE)
    }
    pub fn cli(&mut self, _value: ()) {
        trace!(target: "opcode", "cli");
        self.reg.get_p_mut().clear(flags::INTERRUPT_DISABLE)
    }
    pub fn clv(&mut self, _value: ()) {
        trace!(target: "opcode", "clv");
        self.reg.get_p_mut().clear(flags::OVERFLOW)
    }
    pub fn sec(&mut self, _value: ()) {
        trace!(target: "opcode", "sec");
        self.reg.get_p_mut().set(flags::CARRY)
    }
    pub fn sed(&mut self, _value: ()) {
        trace!(target: "opcode", "sed");
        self.reg.get_p_mut().set(flags::DECIMAL_MODE)
    }
    pub fn sei(&mut self, _value: ()) {
        trace!(target: "opcode", "sei");
        self.reg.get_p_mut().set(flags::INTERRUPT_DISABLE)
    }

    // endregion

    // endregion

    // region Addressing Mode

    // region Write

    // Stack
    // 2    PC     read next instruction byte (and throw it away)
    // 3  $0100,S  push register on stack, decrement S
    pub fn w_stack(&mut self) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.read_pc();
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
    // 2    PC     fetch address, increment PC
    // 3  address  read from effective address
    pub fn w_zero_page(&mut self) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.fetch_into_m();
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
    // 2     PC      fetch address, increment PC
    // 3   address   read from address, add index register to it
    // 4  address+I* read from effective address
    fn w_zero_page_indexed(&mut self, index: u8) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.read_m();
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
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.fetch_into_n();
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
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.fetch_into_n();
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
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.read_m();
                self.reg.write_inc_m_by_x();
                self.reg.write_n(self.reg.get_m().wrapping_add(1));
                Option::None
            }
            cycle::T4 => {
                self.read_m_to_self();
                Option::None
            }
            cycle::T5 => {
                self.read_n_to_self();
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
                self.fetch_into_m();
                self.reg.write_n(self.reg.get_m().wrapping_add(1));
                Option::None
            }
            cycle::T3 => {
                self.read_m_to_self();
                Option::None
            }
            cycle::T4 => {
                self.read_n_to_self();
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
                self.read_pc();
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
                self.read_pc();
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
                Option::Some(self.fetch_pc())
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
                self.read_pc();
                Option::None
            }
            cycle::T3 => {
                self.reg.set_inc_s(1);
                Option::None
            }
            cycle::T4 => {
                self.finish();
                Option::Some(self.read_stack())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Zero Page
    pub fn r_zero_page(&mut self) -> Option<u8> {
        let addr = self.w_zero_page()?;
        Option::Some(self.read_at_m(addr))
    }

    pub fn r_zero_page_x(&mut self) -> Option<u8> {
        let index = self.reg.get_x();
        let addr = self.w_zero_page_indexed(index)?;
        Option::Some(self.read_at_m(addr))
    }

    pub fn r_zero_page_y(&mut self) -> Option<u8> {
        let index = self.reg.get_y();
        let addr = self.w_zero_page_indexed(index)?;
        Option::Some(self.read_at_m(addr))
    }

    // Absolute
    pub fn r_absolute(&mut self) -> Option<u8> {
        let addr = self.w_absolute()?;
        Option::Some(self.read_at_m(addr))
    }

    // Absolute indexed
    // 2     PC      fetch low byte of address, increment PC
    // 3     PC      fetch high byte of address, add index register to low address byte, increment PC
    // 4  address+*  read from effective address, fix the high byte of effective address
    // 5+ address+I  re-read from effective address
    fn r_absolute_indexed(&mut self, index: u8) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.fetch_into_n();
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                let value = self.read_absolute();
                self.reg.set_fix_carry_n();

                if self.reg.get_internal_overflow() == reg::InternalOverflow::None { self.finish() }

                Option::Some(value)
            }
            cycle::T5 => {
                self.finish();
                Option::Some(self.read_absolute())
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
        Option::Some(self.read(addr))
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
                self.fetch_into_m();
                self.reg.write_n(self.reg.get_m().wrapping_add(1));
                Option::None
            }
            cycle::T3 => {
                self.read_m_to_self();
                Option::None
            }
            cycle::T4 => {
                self.read_n_to_self();
                self.reg.write_inc_m_by_y();
                Option::None
            }
            cycle::T5 => {
                let value = self.read_absolute();
                self.reg.set_fix_carry_n();

                if self.reg.get_internal_overflow() == reg::InternalOverflow::None { self.finish() }

                Option::Some(value)
            }
            cycle::T6 => {
                self.finish();
                Option::Some(self.read_absolute())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Relative
    // 2     PC      fetch operand, increment PC. If branch not taken, finish.
    // 3+    PC      Prefetch opcode of next instruction. Add operand to PCL. If there was no overflow, finish.
    // 4+    PC*     Prefetch opcode of next instruction. Fix PCH.
    pub fn relative(&mut self, branch_taken: bool, opcode: &'static str) {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.fetch_into_m();
                if !branch_taken {
                    trace!(target: "opcode", "{}, not taken", opcode);
                    self.finish();
                }
            }
            cycle::T3 => {
                self.prefetch_pc();

                let m = self.reg.get_m() as i8;
                self.reg.write_inc_pcl(m);

                if self.reg.get_internal_overflow() == reg::InternalOverflow::None {
                    trace!(target: "opcode", "{}, taken to 0x{:04x} on t3", opcode, self.reg.get_pc());
                    self.finish();
                }
            }
            cycle::T4 => {
                self.prefetch_pc();
                self.reg.set_fix_carry_pc();
                trace!(target: "opcode", "{}, taken to 0x{:04x} on t4", opcode, self.reg.get_pc());
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
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.read_m_to_n();
                Option::None
            }
            cycle::T4 => {
                self.write_n_to_m();
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
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.read_m();
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                self.read_m_to_n();
                Option::None
            }
            cycle::T5 => {
                self.write_n_to_m();
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
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.fetch_into_n();
                Option::None
            }
            cycle::T4 => {
                self.read_absolute_to_q();
                Option::None
            }
            cycle::T5 => {
                self.write_q_to_absolute();
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
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.fetch_into_n();
                self.reg.write_inc_m(index);
                Option::None
            }
            cycle::T4 => {
                self.read_absolute();
                self.reg.set_fix_carry_n();
                Option::None
            }
            cycle::T5 => {
                self.read_absolute_to_q();
                Option::None
            }
            cycle::T6 => {
                self.write_q_to_absolute();
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
                self.fetch_into_m();
                Option::None
            }
            cycle::T3 => {
                self.read_m();
                self.reg.write_inc_m_by_x();
                self.reg.write_n(self.reg.get_m().wrapping_add(1));
                Option::None
            }
            cycle::T4 => {
                self.read_m_to_self();
                Option::None
            }
            cycle::T5 => {
                self.read_n_to_self();
                Option::None
            }
            cycle::T6 => {
                self.read_absolute_to_q();
                Option::None
            }
            cycle::T7 => {
                self.write_q_to_absolute();
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
                self.fetch_into_m();
                self.reg.write_n(self.reg.get_m().wrapping_add(1));
                Option::None
            }
            cycle::T3 => {
                self.read_m_to_self();
                Option::None
            }
            cycle::T4 => {
                self.read_n_to_self();
                self.reg.write_inc_m_by_y();
                Option::None
            }
            cycle::T5 => {
                self.read_absolute();
                self.reg.set_fix_carry_n();

                Option::None
            }
            cycle::T6 => {
                self.read_absolute_to_q();
                Option::None
            }
            cycle::T7 => {
                self.write_q_to_absolute();
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
