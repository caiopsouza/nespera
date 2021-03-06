use crate::cpu::Cpu;
use crate::cpu::cycle;
use crate::cpu::flags;
use crate::cpu::log::AddrMode;
use crate::cpu::reg;
use crate::utils::bits;

impl Cpu {
    // region Miscellaneous

    // Set current cycle to the last one.
    // It's actually the next to last cycle but this is always incremented later.
    // First cycle is always fetching the opcode.
    fn finish(&mut self) { self.reg.set_next_to_last_cycle() }

    fn read(&mut self, addr: u16) -> u8 {
        self.reg.addr_bus(addr, self.bus.borrow_mut().read_cpu(addr))
    }

    fn peek(&self, addr: u16) -> u8 { self.bus.borrow().peek_cpu(addr) }

    fn write(&mut self, addr: u16, data: u8) {
        self.reg.addr_bus(addr, data);
        self.bus.borrow_mut().write_cpu(addr, data);
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

    // Read from register as an address to the external bus.
    fn read_m(&mut self) -> u8 {
        self.read(u16::from(self.reg.get_m()))
    }

    fn read_at_m(&mut self, addr: u16) -> u8 {
        let data = self.read(addr);
        self.reg.write_m(data);
        data
    }

    fn read_m_to_self(&mut self) {
        let data = self.read(u16::from(self.reg.get_m()));
        self.reg.write_m(data)
    }

    fn read_n_to_self(&mut self) {
        let data = self.read(u16::from(self.reg.get_n()));
        self.reg.write_n(data)
    }

    fn read_m_to_n(&mut self) {
        let data = self.read(u16::from(self.reg.get_m()));
        self.reg.write_n(data)
    }

    fn read_absolute(&mut self) -> u8 {
        self.read(self.reg.get_absolute())
    }

    fn read_absolute_to_q(&mut self) {
        let data = self.read_absolute();
        self.reg.write_q(data)
    }

    // Write from register as an address to the external bus.
    fn write_n_to_m(&mut self) {
        self.write(u16::from(self.reg.get_m()), self.reg.get_n())
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
                self.log.set_mnemonic("KIL");
                self.log.set_mode(AddrMode::Implied);
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
                {
                    let bus = self.bus.borrow();
                    trace!("{}", if bus.nmi { "NMI" } else if bus.irq { "IRC" } else { "BRK" });
                }
                self.log.set_mnemonic("BRK");
                self.log.set_mode(AddrMode::Implied);
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
                // IRQ or NMI clear the Break flag
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
                let pcl = self.read(0xff00 | u16::from(vector));
                self.reg.write_pcl(pcl);
            }
            cycle::T7 => {
                let vector = self.reg.get_m() + 1;
                let pch = self.read(0xff00 | u16::from(vector));
                self.reg.write_pch(pch);

                // Clear the interrupt flags
                {
                    let mut bus = self.bus.borrow_mut();
                    bus.nmi = false;
                    bus.irq = false;
                }

                if !self.interrupting { self.finish() }
            }
            cycle::T8 | cycle::T9 => {}
            cycle::T10 => {
                self.interrupting = false;
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

    pub fn nop(&mut self, _: ()) -> (&'static str, u8) { ("NOP", 0) }

    pub fn mop(&mut self, _: ()) -> (&'static str, u8) {
        self.log.set_unofficial(true);
        ("NOP", 0)
    }

    pub fn dop(&mut self, _: u8) -> (&'static str, u8) {
        self.log.set_unofficial(true);
        ("NOP", 0)
    }

    // endregion

    // region Load

    pub fn lda(&mut self, data: u8) -> (&'static str, u8) {
        self.reg.write_a(data);
        ("LDA", data)
    }

    pub fn ldx(&mut self, data: u8) -> (&'static str, u8) {
        self.reg.write_x(data);
        ("LDX", data)
    }

    pub fn ldy(&mut self, data: u8) -> (&'static str, u8) {
        self.reg.write_y(data);
        ("LDY", data)
    }

    pub fn lax(&mut self, data: u8) -> (&'static str, u8) {
        self.reg.write_a(data);
        self.reg.write_x(data);

        self.log.set_unofficial(true);
        ("LAX", data)
    }

    // endregion

    // region Store

    pub fn sta(&mut self, addr: u16) -> (&'static str, u8) {
        let data = self.reg.get_a();
        self.write(addr, data);
        ("STA", data)
    }

    pub fn stx(&mut self, addr: u16) -> (&'static str, u8) {
        let data = self.reg.get_x();
        self.write(addr, data);
        ("STX", data)
    }

    pub fn sty(&mut self, addr: u16) -> (&'static str, u8) {
        let data = self.reg.get_y();
        self.write(addr, data);
        ("STY", data)
    }

    pub fn shx(&mut self, addr: u16) -> (&'static str, u8) {
        let x = self.reg.get_x();
        let data = x & (bits::high(addr) + 1);
        self.write(addr, data);

        self.log.set_unofficial(true);
        ("SHX", data)
    }

    pub fn shy(&mut self, addr: u16) -> (&'static str, u8) {
        let y = self.reg.get_y();
        let data = y & (bits::high(addr) + 1);
        self.write(addr, data);

        self.log.set_unofficial(true);
        ("SHY", data)
    }

    pub fn ahx(&mut self, (addr, _): (u16, u8)) -> (&'static str, u8) {
        let a = self.reg.get_a();
        let x = self.reg.get_x();

        let data = a & x & (bits::high(addr) + 1);
        self.write(addr, data);

        self.log.set_unofficial(true);
        ("AHX", data)
    }

    pub fn tas(&mut self, addr: u16) -> (&'static str, u8) {
        let a = self.reg.get_a();
        let x = self.reg.get_x();
        let s = a & x;
        self.reg.set_s(s);

        let data = s & (bits::high(addr) + 1);
        self.write(addr, data);

        self.log.set_unofficial(true);
        ("TAS", data)
    }

    pub fn las(&mut self, data: u8) -> (&'static str, u8) {
        let s = self.reg.get_s();
        let data = s & data;

        self.reg.write_a(data);
        self.reg.write_x(data);
        self.reg.set_s(data);

        self.log.set_unofficial(true);
        ("LAS", data)
    }

    // endregion

    // region Transfer

    pub fn tax(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_a();
        self.reg.write_x(data);
        ("TAX", data)
    }

    pub fn tay(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_a();
        self.reg.write_y(data);
        ("TAY", data)
    }

    pub fn txa(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_x();
        self.reg.write_a(data);
        ("TXA", data)
    }

    pub fn tya(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_y();
        self.reg.write_a(data);
        ("TYA", data)
    }

    pub fn oam(&mut self) {
        let cycle = self.reg.get_cycle();
        let index = (cycle - 2) / 2;

        if cycle == 2 { trace!(target: "opcode", "oam, 0x{:04x}", self.bus.borrow().ppu.oam_source); }

        if cycle % 2 == 0 {
            // Read cycle
            let mut bus = self.bus.borrow_mut();
            let addr = bus.ppu.oam_source | index;
            let data = bus.read_cpu(addr);
            self.reg.set_m(data);
        } else {
            // Write cycle
            {
                let mut bus = self.bus.borrow_mut();
                let data = self.reg.get_m();
                bus.write_cpu(0x2004, data);
            }

            if index == 255 {
                self.oam_transferring = false;
                self.bus.borrow_mut().ppu.oam_transfer = false;
                self.reg.set_last_cycle();
            }
        }
    }

    // endregion

    // region Stack

    pub fn tsx(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_s();
        self.reg.write_x(data);
        ("TSX", data)
    }

    pub fn txs(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_x();
        self.reg.write_s(data);
        ("TXS", data)
    }

    pub fn pha(&mut self, addr: u16) -> (&'static str, u8) {
        let data = self.reg.get_a();
        self.write(addr, data);
        ("PHA", data)
    }

    pub fn php(&mut self, addr: u16) -> (&'static str, u8) {
        // Break Command and Unused are always set when pushing
        let data = self.reg.get_p() | flags::BREAK_COMMAND | flags::UNUSED;
        let data = data.as_u8();
        self.write(addr, data);

        ("PHP", data)
    }

    pub fn pla(&mut self, data: u8) -> (&'static str, u8) {
        self.reg.write_a(data);
        ("PLA", data)
    }

    pub fn plp(&mut self, data: u8) -> (&'static str, u8) {
        // Break Command and Unused are ignored when pulling
        let mut data = flags::Flags(data);
        data.copy(self.reg.get_p(), flags::BREAK_COMMAND | flags::UNUSED);
        self.reg.write_p(data);

        ("PLP", data.into())
    }

    // endregion

    // region Logic

    pub fn and(&mut self, data: u8) -> (&'static str, u8) {
        let data = data & self.reg.get_a();
        self.reg.write_a(data);
        ("AND", data)
    }

    pub fn eor(&mut self, data: u8) -> (&'static str, u8) {
        let data = data ^ self.reg.get_a();
        self.reg.write_a(data);
        ("EOR", data)
    }

    pub fn ora(&mut self, data: u8) -> (&'static str, u8) {
        let data = data | self.reg.get_a();
        self.reg.write_a(data);
        ("ORA", data)
    }

    pub fn bit(&mut self, data: u8) -> (&'static str, u8) {
        self.reg.write_bit_test(data);
        ("BIT", data)
    }

    pub fn anc(&mut self, data: u8) -> (&'static str, u8) {
        let data = data & self.reg.get_a();
        self.reg.write_a(data);

        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, p.get_negative());

        self.log.set_unofficial(true);
        ("ANC", data)
    }

    pub fn xaa(&mut self, data: u8) -> (&'static str, u8) {
        trace!(target: "opcode", "xaa, data: 0x{:02x}", data);
        let a = self.reg.get_a();

        // XAA has analogic behaviour.
        // "Magic" defines which bits of A will be used in the result and can vary wildly.
        // Below values happen over 98% of the time.
        let magic = if (a & 0x01) == 0 { 0xfe } else { 0xff };

        let data = (a | magic) & self.reg.get_x() & data;
        self.reg.write_a(data);

        self.log.set_unofficial(true);
        ("XAA", data)
    }

    pub fn sax(&mut self, addr: u16) -> (&'static str, u8) {
        let data = self.reg.get_a() & self.reg.get_x();
        self.write(addr, data);

        self.log.set_unofficial(true);
        ("SAX", data)
    }

    // endregion

    // region Arithmetic

    pub fn adc(&mut self, data: u8) -> (&'static str, u8) {
        let a = self.reg.get_a();
        let p = self.reg.get_p_mut();

        let res = u16::from(a) + u16::from(data) + u16::from(p.get_carry());

        p.change_zero_negative(res as u8);

        // When adding, carry happens if bit 8 is set
        p.change(flags::CARRY, (res & 0x0100_u16) != 0);

        // Overflow happens when the sign of the addends is the same and differs from the sign of the sum
        p.change(
            flags::OVERFLOW,
            (!(a ^ data) & (a ^ res as u8) & 0x80) != 0,
        );

        self.reg.write_a(res as u8);
        ("ADC", data)
    }

    pub fn sbc(&mut self, value: u8) -> (&'static str, u8) {
        // Since you should subtract (1 - carry) inverting the value
        // has the same effect as a two's complement after the carry is added
        // Carry is inverted.
        ("SBC", self.adc(!value).1)
    }

    // Unofficial SBC
    pub fn sbn(&mut self, value: u8) -> (&'static str, u8) {
        self.log.set_unofficial(true);
        self.sbc(value)
    }

    // endregion

    // region Comparison

    pub fn cmp(&mut self, data: u8) -> (&'static str, u8) {
        let source = self.reg.get_a();
        self.reg.get_p_mut().change_cmp(source, data);
        ("CMP", data)
    }

    pub fn cpx(&mut self, data: u8) -> (&'static str, u8) {
        let source = self.reg.get_x();
        self.reg.get_p_mut().change_cmp(source, data);
        ("CPX", data)
    }

    pub fn cpy(&mut self, data: u8) -> (&'static str, u8) {
        let source = self.reg.get_y();
        self.reg.get_p_mut().change_cmp(source, data);
        ("CPY", data)
    }

    pub fn dcp(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let source = self.reg.get_a();
        let data = data.wrapping_sub(1);

        self.write(addr, data);
        self.reg.get_p_mut().change_cmp(source, data);

        self.log.set_unofficial(true);
        ("DCP", data)
    }

    pub fn axs(&mut self, data: u8) -> (&'static str, u8) {
        let v = self.reg.get_p().get_overflow();

        let x = self.reg.get_a() & self.reg.get_x();
        self.reg.write_x(x.wrapping_sub(data));

        let p = self.reg.get_p_mut();
        p.change_cmp(x, data);

        // Overflow flag in not affected, so change to the old value.
        p.change(flags::OVERFLOW, v);

        self.log.set_unofficial(true);
        ("AXS", p.as_u8())
    }

    // endregion

    // region Increment

    pub fn inc(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let data = data.wrapping_add(1);
        self.reg.get_p_mut().change_zero_negative(data);
        self.write(addr, data);
        ("INC", data)
    }

    pub fn inx(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_x().wrapping_add(1);
        self.reg.write_x(data);
        ("INX", data)
    }

    pub fn iny(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_y().wrapping_add(1);
        self.reg.write_y(data);
        ("INY", data)
    }

    pub fn isc(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let data = data.wrapping_add(1);

        self.sbc(data);
        self.write(addr, data);

        self.log.set_unofficial(true);
        ("ISC", data)
    }

    // endregion

    // region Decrement

    pub fn dec(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let data = data.wrapping_sub(1);
        self.reg.get_p_mut().change_zero_negative(data);
        self.write(addr, data);
        ("DEC", data)
    }

    pub fn dex(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_x().wrapping_sub(1);
        self.reg.write_x(data);
        ("DEX", data)
    }

    pub fn dey(&mut self, _: ()) -> (&'static str, u8) {
        let data = self.reg.get_y().wrapping_sub(1);
        self.reg.write_y(data);
        ("DEY", data)
    }

    // endregion

    // region Shift

    fn shift(&mut self, addr: u16, res: u8, condition: bool) {
        let p = self.reg.get_p_mut();

        p.change(flags::CARRY, condition);
        p.change_zero_negative(res);

        self.write(addr, res);
    }

    pub fn asl(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let res = data << 1;
        self.shift(addr,
                   res,
                   (data & 0b1000_0000) != 0);
        ("ASL", res)
    }

    pub fn asl_acc(&mut self, data: u8) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, (data & 0b1000_0000) != 0);

        let data = data << 1;
        self.reg.write_a(data);

        ("ASL", data)
    }

    pub fn lsr(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let res = data >> 1;
        self.shift(addr,
                   res,
                   (data & 0b0000_0001) != 0);
        ("LSR", res)
    }

    pub fn lsr_acc(&mut self, data: u8) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.change(flags::CARRY, (data & 0b0000_0001) != 0);

        let data = data >> 1;
        self.reg.write_a(data);
        ("LSR", data)
    }

    pub fn rol(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let carry = self.reg.get_p().get_carry();
        let res = (data << 1) | (carry as u8);
        self.shift(addr,
                   res,
                   (data & 0b1000_0000) != 0);
        ("ROL", res)
    }

    pub fn rol_acc(&mut self, data: u8) -> (&'static str, u8) {
        trace!(target: "opcode", "rol, data: 0x{:02x}", data);
        let p = self.reg.get_p_mut();
        let carry = p.get_carry();

        p.change(flags::CARRY, (data & 0b1000_0000) != 0);

        let data = (data << 1) | (carry as u8);
        self.reg.write_a(data);

        ("ROL", data)
    }

    pub fn ror(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let carry = self.reg.get_p().get_carry();
        let res = (data >> 1) | ((carry as u8) << 7);
        self.shift(addr,
                   res,
                   (data & 0b0000_0001) != 0);
        ("ROR", res)
    }

    pub fn ror_acc(&mut self, data: u8) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        let carry = p.get_carry();
        p.change(flags::CARRY, (data & 0b0000_0001) != 0);

        let data = (data >> 1) | ((carry as u8) << 7);
        self.reg.write_a(data);
        ("ROR", data)
    }

    pub fn slo(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let data = self.asl((addr, data)).1;
        let data = self.ora(data).1;

        self.log.set_unofficial(true);
        ("SLO", data)
    }

    pub fn sre(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let data = self.lsr((addr, data)).1;
        let data = self.eor(data).1;

        self.log.set_unofficial(true);
        ("SRE", data)
    }

    pub fn rla(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let data = self.rol((addr, data)).1;
        let data = self.and(data).1;

        self.log.set_unofficial(true);
        ("RLA", data)
    }

    pub fn alr(&mut self, data: u8) -> (&'static str, u8) {
        self.and(data);
        let a = self.reg.get_a();
        let res = self.lsr_acc(a).1;

        self.log.set_unofficial(true);
        ("ALR", res)
    }

    pub fn rra(&mut self, (addr, data): (u16, u8)) -> (&'static str, u8) {
        let data = self.ror((addr, data)).1;
        let data = self.adc(data).1;

        self.log.set_unofficial(true);
        ("RRA", data)
    }

    pub fn arr(&mut self, data: u8) -> (&'static str, u8) {
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

        self.log.set_unofficial(true);
        ("ARR", res)
    }

    // endregion

    // region Jump

    // Jump to absolute address
    // 2    PC     fetch low address byte, increment PC
    // 3    PC     copy low address byte to PCL, fetch high address byte to PCH
    pub fn jmp_absolute(&mut self) {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.fetch_into_m();
            }
            cycle::T3 => {
                self.fetch_into_n();

                let pc = self.reg.get_absolute();
                self.reg.write_pc(pc);

                self.log.set_mnemonic("JMP");
                self.log.set_mode(AddrMode::Direct(pc));

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
                // Logs the correct address, respecting cross page.
                let pcl = self.reg.get_q();
                let pch = self.peek(self.reg.get_absolute().wrapping_add(1));
                let addr = bits::word(pch, pcl);
                self.log.set_mnemonic("JMP");
                self.log.set_mode(AddrMode::Indirect(self.reg.get_absolute(), addr));

                // A bug on the hardware makes it not respect page cross. This is emulate here.
                self.reg.write_inc_m(1);
                self.read_absolute_to_q();
                self.reg.write_pcl_pch(pcl, self.reg.get_q());

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

                self.log.set_mnemonic("JSR");
                self.log.set_mode(AddrMode::Direct(self.reg.get_pc()));
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

                self.log.set_mnemonic("RTI");
                self.log.set_mode(AddrMode::Implied);
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

                self.log.set_mnemonic("RTS");
                self.log.set_mode(AddrMode::Implied);
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // region Branch

    pub fn bcs(&mut self) { self.relative(self.reg.get_p().get_carry(), "BCS") }
    pub fn bcc(&mut self) { self.relative(!self.reg.get_p().get_carry(), "BCC") }
    pub fn beq(&mut self) { self.relative(self.reg.get_p().get_zero(), "BEQ") }
    pub fn bne(&mut self) { self.relative(!self.reg.get_p().get_zero(), "BNE") }
    pub fn bmi(&mut self) { self.relative(self.reg.get_p().get_negative(), "BMI") }
    pub fn bpl(&mut self) { self.relative(!self.reg.get_p().get_negative(), "BPL") }
    pub fn bvs(&mut self) { self.relative(self.reg.get_p().get_overflow(), "BVS") }
    pub fn bvc(&mut self) { self.relative(!self.reg.get_p().get_overflow(), "BVC") }

    // endregion

    // region Status flags

    pub fn clc(&mut self, _: ()) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.clear(flags::CARRY);
        ("CLC", p.as_u8())
    }

    pub fn cld(&mut self, _: ()) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.clear(flags::DECIMAL_MODE);
        ("CLD", p.as_u8())
    }

    pub fn cli(&mut self, _: ()) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.clear(flags::INTERRUPT_DISABLE);
        ("CLI", p.as_u8())
    }

    pub fn clv(&mut self, _: ()) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.clear(flags::OVERFLOW);
        ("CLV", p.as_u8())
    }

    pub fn sec(&mut self, _: ()) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.set(flags::CARRY);
        ("SEC", p.as_u8())
    }

    pub fn sed(&mut self, _: ()) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.set(flags::DECIMAL_MODE);
        ("SED", p.as_u8())
    }

    pub fn sei(&mut self, _: ()) -> (&'static str, u8) {
        let p = self.reg.get_p_mut();
        p.set(flags::INTERRUPT_DISABLE);
        ("SEI", p.as_u8())
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
                None
            }
            cycle::T3 => {
                self.log.set_mode(AddrMode::Implied);
                self.reg.set_inc_s(-1);
                self.finish();
                Some(self.reg.get_next_stack_addr())
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
                None
            }
            cycle::T3 => {
                let m = self.reg.get_m();
                self.log.set_mode(AddrMode::ZeroPage(m, self.peek(u16::from(m))));
                self.finish();
                Some(u16::from(m))
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
                None
            }
            cycle::T3 => {
                self.read_m();
                self.reg.write_inc_m(index);
                None
            }
            cycle::T4 => {
                self.finish();
                Some(u16::from(self.reg.get_m()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn w_zero_page_x(&mut self) -> Option<u16> {
        let index = self.reg.get_x();
        let addr = self.w_zero_page_indexed(index)?;
        self.log.set_mode(AddrMode::ZeroPageX(self.peek(self.reg.get_pc() - 1), addr as u8, self.peek(addr)));
        Some(addr)
    }

    pub fn w_zero_page_y(&mut self) -> Option<u16> {
        let index = self.reg.get_y();
        let addr = self.w_zero_page_indexed(index)?;
        self.log.set_mode(AddrMode::ZeroPageY(self.peek(self.reg.get_pc() - 1), addr as u8, self.peek(addr)));
        Some(addr)
    }

    // Absolute
    // 2    PC     fetch low byte of address, increment PC
    // 3    PC     fetch high byte of address, increment PC
    // 4  address  read from effective address
    pub fn w_absolute(&mut self) -> Option<u16> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.fetch_into_m();
                None
            }
            cycle::T3 => {
                self.fetch_into_n();
                None
            }
            cycle::T4 => {
                self.finish();
                let addr = self.reg.get_absolute();
                self.log.set_mode(AddrMode::Absolute(addr, self.peek(addr)));
                Some(addr)
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
                None
            }
            cycle::T3 => {
                self.fetch_into_n();
                self.reg.write_inc_m(index);
                None
            }
            cycle::T4 => {
                self.reg.get_absolute();
                self.reg.set_fix_carry_n();
                None
            }
            cycle::T5 => {
                self.finish();
                Some(self.reg.get_absolute())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn w_absolute_x(&mut self) -> Option<u16> {
        let index = self.reg.get_x();
        let addr = self.w_absolute_indexed(index)?;
        self.log.set_mode(AddrMode::AbsoluteX(
            bits::word(
                self.peek(self.reg.get_pc() - 1),
                self.peek(self.reg.get_pc() - 2)),
            addr,
            self.peek(addr)));
        Some(addr)
    }

    pub fn w_absolute_y(&mut self) -> Option<u16> {
        let index = self.reg.get_y();
        let addr = self.w_absolute_indexed(index)?;
        self.log.set_mode(AddrMode::AbsoluteY(
            bits::word(
                self.peek(self.reg.get_pc() - 1),
                self.peek(self.reg.get_pc() - 2)),
            addr,
            self.peek(addr)));
        Some(addr)
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
                None
            }
            cycle::T3 => {
                self.read_m();
                self.reg.write_inc_m_by_x();
                self.reg.write_n(self.reg.get_m().wrapping_add(1));
                None
            }
            cycle::T4 => {
                self.read_m_to_self();
                None
            }
            cycle::T5 => {
                self.read_n_to_self();
                None
            }
            cycle::T6 => {
                let addr = self.reg.get_absolute();
                let op0 = self.peek(self.reg.get_pc() - 1);

                self.log.set_mode(AddrMode::IndirectX(
                    op0,
                    op0.wrapping_add(self.reg.get_x()),
                    addr,
                    self.peek(addr),
                ));

                self.finish();
                Some(addr)
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
                None
            }
            cycle::T3 => {
                self.read_m_to_self();
                None
            }
            cycle::T4 => {
                self.read_n_to_self();
                self.reg.write_inc_m_by_y();
                None
            }
            cycle::T5 => {
                self.reg.get_absolute();
                self.reg.set_fix_carry_n();
                None
            }
            cycle::T6 => {
                let addr = self.reg.get_absolute();
                let table = addr.wrapping_sub(u16::from(self.reg.get_y()));
                self.log.set_mode(AddrMode::IndirectY(
                    self.peek(self.reg.get_pc() - 1),
                    table,
                    addr,
                    self.peek(addr),
                ));

                self.finish();
                Some(addr)
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
                self.log.set_mode(AddrMode::Implied);

                self.read_pc();
                self.finish();

                Some(())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Accumulator
    // 2    PC     read next instruction byte (and throw it away)
    pub fn accumulator(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.log.set_mode(AddrMode::Accumulator);
                self.read_pc();
                self.finish();
                Some(self.reg.read_a())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Immediate
    // 2    PC     fetch value, increment PC
    pub fn immediate(&mut self) -> Option<u8> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.log.set_mode(AddrMode::Immediate(self.peek(self.reg.get_pc())));
                self.finish();
                Some(self.fetch_pc())
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
                None
            }
            cycle::T3 => {
                self.reg.set_inc_s(1);
                None
            }
            cycle::T4 => {
                self.log.set_mode(AddrMode::Implied);
                self.finish();
                Some(self.read_stack())
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // Zero Page
    pub fn r_zero_page(&mut self) -> Option<u8> {
        let addr = self.w_zero_page()?;
        Some(self.read_at_m(addr))
    }

    pub fn r_zero_page_x(&mut self) -> Option<u8> {
        let index = self.reg.get_x();
        let addr = self.w_zero_page_indexed(index)?;
        let data = self.read_at_m(addr);

        self.log.set_mode(AddrMode::ZeroPageX(self.peek(self.reg.get_pc() - 1), addr as u8, data));

        Some(data)
    }

    pub fn r_zero_page_y(&mut self) -> Option<u8> {
        let index = self.reg.get_y();
        let addr = self.w_zero_page_indexed(index)?;
        let data = self.read_at_m(addr);

        self.log.set_mode(AddrMode::ZeroPageY(self.peek(self.reg.get_pc() - 1), addr as u8, data));

        Some(data)
    }

    // Absolute
    pub fn r_absolute(&mut self) -> Option<u8> {
        let addr = self.w_absolute()?;
        Some(self.read_at_m(addr))
    }

    // Absolute indexed
    // 2     PC      fetch low byte of address, increment PC
    // 3     PC      fetch high byte of address, add index register to low address byte, increment PC
    // 4  address+*  read from effective address, fix the high byte of effective address
    // 5+ address+I  re-read from effective address
    fn r_absolute_indexed(&mut self, index: u8) -> Option<(u16, u8)> {
        match self.reg.get_cycle() {
            cycle::T2 => {
                self.fetch_into_m();
                None
            }
            cycle::T3 => {
                self.fetch_into_n();
                self.reg.write_inc_m(index);
                None
            }
            cycle::T4 => {
                let data = self.read_absolute();
                self.reg.set_fix_carry_n();

                if self.reg.get_internal_overflow() == reg::InternalOverflow::None {
                    self.finish();
                    Some((self.reg.get_absolute(), data))
                } else { None }
            }
            cycle::T5 => {
                self.finish();
                Some((self.reg.get_absolute(), self.read_absolute()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn r_absolute_x(&mut self) -> Option<u8> {
        let index = self.reg.get_x();
        let res = self.r_absolute_indexed(index)?;

        self.log.set_mode(AddrMode::AbsoluteX(
            bits::word(
                self.peek(self.reg.get_pc() - 1),
                self.peek(self.reg.get_pc() - 2)),
            res.0,
            res.1));

        Some(res.1)
    }

    pub fn r_absolute_y(&mut self) -> Option<u8> {
        let index = self.reg.get_y();
        let res = self.r_absolute_indexed(index)?;

        self.log.set_mode(AddrMode::AbsoluteY(
            bits::word(
                self.peek(self.reg.get_pc() - 1),
                self.peek(self.reg.get_pc() - 2)),
            res.0,
            res.1));

        Some(res.1)
    }

    // Indexed indirect by X
    pub fn r_indirect_x(&mut self) -> Option<u8> {
        let addr = self.w_indirect_x()?;
        Some(self.read(addr))
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
                None
            }
            cycle::T3 => {
                self.read_m_to_self();
                None
            }
            cycle::T4 => {
                self.read_n_to_self();

                let absolute = self.reg.get_absolute();
                let addr = absolute.wrapping_add(u16::from(self.reg.get_y()));
                self.log.set_mode(AddrMode::IndirectY(
                    self.peek(self.reg.get_pc() - 1),
                    absolute,
                    addr,
                    self.peek(addr),
                ));

                self.reg.write_inc_m_by_y();
                None
            }
            cycle::T5 => {
                let data = self.read_absolute();
                self.reg.set_fix_carry_n();

                if self.reg.get_internal_overflow() == reg::InternalOverflow::None {
                    self.finish();
                    Some(data)
                } else { None }
            }
            cycle::T6 => {
                self.finish();
                Some(self.read_absolute())
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
                self.log.set_mnemonic(opcode);

                if !branch_taken {
                    //let pc = self.reg.get_pc();
                    //let pc = (bits::high_word(pc)) | bits::low_word(pc).wrapping_add(u16::from(self.reg.get_m()));

                    let m = i16::from(self.reg.get_m() as i8) as u16;
                    self.log.set_mode(AddrMode::Relative(self.reg.get_m(), self.reg.get_pc().wrapping_add(m)));
                    self.finish();
                }
            }
            cycle::T3 => {
                self.prefetch_pc();

                let m = self.reg.get_m() as i8;
                self.reg.write_inc_pcl(m);

                if self.reg.get_internal_overflow() == reg::InternalOverflow::None {
                    self.log.set_mode(AddrMode::Relative(self.reg.get_m(), self.reg.get_pc()));
                    self.finish();
                }
            }
            cycle::T4 => {
                self.prefetch_pc();
                self.reg.set_fix_carry_pc();
                self.log.set_mode(AddrMode::Relative(self.reg.get_m(), self.reg.get_pc()));
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
                None
            }
            cycle::T3 => {
                self.read_m_to_n();
                None
            }
            cycle::T4 => {
                self.write_n_to_m();
                None
            }
            cycle::T5 => {
                let m = self.reg.get_m();
                let n = self.reg.get_n();
                self.log.set_mode(AddrMode::ZeroPage(m, n));
                self.finish();
                Some((u16::from(m), n))
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
                None
            }
            cycle::T3 => {
                self.read_m();
                self.reg.write_inc_m(index);
                None
            }
            cycle::T4 => {
                self.read_m_to_n();
                None
            }
            cycle::T5 => {
                self.write_n_to_m();
                None
            }
            cycle::T6 => {
                self.finish();
                Some((u16::from(self.reg.get_m()), self.reg.get_n()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn rw_zero_page_x(&mut self) -> Option<(u16, u8)> {
        let index = self.reg.get_x();
        let res = self.rw_zero_page_indexed(index)?;
        self.log.set_mode(AddrMode::ZeroPageX(self.peek(self.reg.get_pc() - 1), res.0 as u8, res.1));
        Some(res)
    }

    pub fn rw_zero_page_y(&mut self) -> Option<(u16, u8)> {
        let index = self.reg.get_y();
        let res = self.rw_zero_page_indexed(index)?;
        self.log.set_mode(AddrMode::ZeroPageY(self.peek(self.reg.get_pc() - 1), res.0 as u8, res.1));
        Some(res)
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
                None
            }
            cycle::T3 => {
                self.fetch_into_n();
                None
            }
            cycle::T4 => {
                self.read_absolute_to_q();
                None
            }
            cycle::T5 => {
                self.write_q_to_absolute();
                None
            }
            cycle::T6 => {
                let res = (self.reg.get_absolute(), self.reg.get_q());
                self.log.set_mode(AddrMode::Absolute(res.0, res.1));
                self.finish();
                Some(res)
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
                None
            }
            cycle::T3 => {
                self.fetch_into_n();
                self.reg.write_inc_m(index);
                None
            }
            cycle::T4 => {
                self.read_absolute();
                self.reg.set_fix_carry_n();
                None
            }
            cycle::T5 => {
                self.read_absolute_to_q();
                None
            }
            cycle::T6 => {
                self.write_q_to_absolute();
                None
            }
            cycle::T7 => {
                self.finish();
                Some((self.reg.get_absolute(), self.reg.get_q()))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    pub fn rw_absolute_x(&mut self) -> Option<(u16, u8)> {
        let index = self.reg.get_x();
        let res = self.rw_absolute_indexed(index)?;

        self.log.set_mode(AddrMode::AbsoluteX(
            bits::word(
                self.peek(self.reg.get_pc() - 1),
                self.peek(self.reg.get_pc() - 2)),
            res.0,
            res.1));

        Some(res)
    }

    pub fn rw_absolute_y(&mut self) -> Option<(u16, u8)> {
        let index = self.reg.get_y();
        let res = self.rw_absolute_indexed(index)?;

        self.log.set_mode(AddrMode::AbsoluteY(
            bits::word(
                self.peek(self.reg.get_pc() - 1),
                self.peek(self.reg.get_pc() - 2)),
            res.0,
            res.1));

        Some(res)
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
                None
            }
            cycle::T3 => {
                self.read_m();
                self.reg.write_inc_m_by_x();
                self.reg.write_n(self.reg.get_m().wrapping_add(1));
                None
            }
            cycle::T4 => {
                self.read_m_to_self();
                None
            }
            cycle::T5 => {
                self.read_n_to_self();
                None
            }
            cycle::T6 => {
                self.read_absolute_to_q();
                None
            }
            cycle::T7 => {
                self.write_q_to_absolute();
                None
            }
            cycle::T8 => {
                let addr = self.reg.get_absolute();
                let op0 = self.peek(self.reg.get_pc() - 1);
                let data = self.reg.get_q();

                self.log.set_mode(AddrMode::IndirectX(
                    op0,
                    op0.wrapping_add(self.reg.get_x()),
                    addr,
                    data,
                ));

                self.finish();
                Some((addr, data))
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
                None
            }
            cycle::T3 => {
                self.read_m_to_self();
                None
            }
            cycle::T4 => {
                self.read_n_to_self();
                self.reg.write_inc_m_by_y();
                None
            }
            cycle::T5 => {
                self.read_absolute();
                self.reg.set_fix_carry_n();
                None
            }
            cycle::T6 => {
                self.read_absolute_to_q();
                None
            }
            cycle::T7 => {
                self.write_q_to_absolute();
                None
            }
            cycle::T8 => {
                let addr = self.reg.get_absolute();
                let data = self.reg.get_q();

                // Logs the correct address, respecting cross page.
                let op0 = self.peek(self.reg.get_pc() - 1);

                self.log.set_mode(AddrMode::IndirectY(
                    op0,
                    addr.wrapping_sub(u16::from(self.reg.get_y())),
                    addr,
                    data,
                ));

                self.finish();
                Some((addr, data))
            }
            _ => unimplemented!("Shouldn't reach cycle {}", self.reg.get_cycle()),
        }
    }

    // endregion

    // endregion
}
