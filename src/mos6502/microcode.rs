use crate::mos6502::bus::Bus;
use crate::mos6502::reg::Reg;

macro_rules! run {
    () => {
        use super::super::finish::*;
        use crate::mos6502::bus::Bus;
        use crate::mos6502::cycle;
        use crate::mos6502::reg::Reg;

        pub fn run(reg: &mut Reg, bus: &mut Bus) {
            match reg.get_cycle() {
                cycle::T2 => { reg.set_next_cycle(); t2(reg, bus); },
                cycle::T3 => { reg.set_next_cycle(); t3(reg, bus); },
                cycle::T4 => { reg.set_next_cycle(); t4(reg, bus); },
                cycle::T5 => { reg.set_next_cycle(); t5(reg, bus); },
                cycle::T6 => { reg.set_next_cycle(); t6(reg, bus); },
                cycle::T7 => { reg.set_next_cycle(); t7(reg, bus); },
                cycle::T8 => { reg.set_next_cycle(); t8(reg, bus); },
                n => {
                    reg.set_last_cycle();
                    debug_assert!(false, "Should've not reached cycle {}", n.0);
                },
            }
        }
    };

    ( $ ( $cycle: ident => $step:path ),* ) => {
        $ ( pub fn $cycle(reg: &mut Reg, bus: &mut Bus) { $step(reg, bus); } )*
        run!();
    };
}

// Finishes the instruction
pub mod finish {
    use super::*;

    pub fn t2(reg: &mut Reg, _: &mut Bus) {
        debug_assert!(false, "Should've not reached t2.");
        reg.set_last_cycle();
    }

    pub fn t3(reg: &mut Reg, _: &mut Bus) {
        debug_assert!(false, "Should've not reached t3.");
        reg.set_last_cycle();
    }

    pub fn t4(reg: &mut Reg, _: &mut Bus) {
        debug_assert!(false, "Should've not reached t4.");
        reg.set_last_cycle();
    }

    pub fn t5(reg: &mut Reg, _: &mut Bus) {
        debug_assert!(false, "Should've not reached t5.");
        reg.set_last_cycle();
    }

    pub fn t6(reg: &mut Reg, _: &mut Bus) {
        debug_assert!(false, "Should've not reached t6.");
        reg.set_last_cycle();
    }

    pub fn t7(reg: &mut Reg, _: &mut Bus) {
        debug_assert!(false, "Should've not reached t7.");
        reg.set_last_cycle();
    }

    pub fn t8(reg: &mut Reg, _: &mut Bus) {
        debug_assert!(false, "Should've not reached t8.");
        reg.set_last_cycle();
    }
}

// Read operations
mod read {
    use super::*;

    pub mod immediate {
        use super::*;

        pub fn t2(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.fetch_into_m(bus) }
    }

    pub mod zero_page {
        use super::*;

        pub fn t2(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.fetch_into_m(bus) }

        pub fn t3(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.peek_m(bus) }
    }

    pub mod zero_page_x {
        use super::*;

        pub fn t2(reg: &mut Reg, bus: &mut Bus) {
            reg.fetch_into_m(bus);
            reg.write_inc_m_by_x();
        }

        pub fn t3(reg: &mut Reg, bus: &mut Bus) -> u8 { super::zero_page::t3(reg, bus) }

        pub fn t4(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.peek_m(bus) }
    }

    pub mod absolute {
        use super::*;

        pub fn t2(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.fetch_into_m(bus) }

        pub fn t3(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.fetch_into_n(bus) }

        pub fn t4(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.peek_absolute(bus) }
    }

    pub mod absolute_x {
        use super::*;

        pub fn t2(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.fetch_into_m(bus) }

        pub fn t3(reg: &mut Reg, bus: &mut Bus) {
            reg.fetch_into_n(bus);
            reg.write_inc_m_by_x();
        }

        pub fn t4(reg: &mut Reg, bus: &mut Bus) -> u8 {
            let internal_overflow = reg.get_internal_overflow();

            let res = reg.peek_absolute(bus);
            reg.set_fix_carry_n();

            if !internal_overflow { reg.set_last_cycle() }

            res
        }

        pub fn t5(reg: &mut Reg, bus: &mut Bus) -> u8 {
            let res = reg.peek_absolute(bus);
            reg.set_last_cycle();
            res
        }
    }

    pub mod absolute_y {
        use super::*;

        pub fn t2(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.fetch_into_m(bus) }

        pub fn t3(reg: &mut Reg, bus: &mut Bus) {
            reg.fetch_into_n(bus);
            reg.write_inc_m_by_y();
        }

        pub fn t4(reg: &mut Reg, bus: &mut Bus) {
            let overflow_operand = reg.get_internal_overflow();

            reg.peek_absolute(bus);
            reg.set_fix_carry_n();

            if overflow_operand {
                reg.set_next_cycle()
            } else {
                reg.set_last_cycle()
            }
        }

        pub fn t5(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.peek_absolute(bus) }
    }

    pub mod indirect_x {
        use super::*;

        pub fn t2(reg: &mut Reg, bus: &mut Bus) -> u8 { reg.fetch_into_m(bus) }

        pub fn t3(reg: &mut Reg, bus: &mut Bus) {
            reg.peek_m(bus);
            reg.write_inc_m_by_x();
            let m = reg.read_m();
            reg.write_n(m.wrapping_add(1));
        }

        pub fn t4(reg: &mut Reg, bus: &mut Bus) { reg.peek_m_at_self(bus) }

        pub fn t5(reg: &mut Reg, bus: &mut Bus) { reg.peek_n_at_self(bus) }

        pub fn t6(reg: &mut Reg, bus: &mut Bus) -> u8 {
            let res = reg.peek_absolute(bus);
            reg.set_last_cycle();
            res
        }
    }
}

// Stops the cycle from advancing, effectively crashing the CPU.
pub mod kil {
    use super::*;

    pub fn run(_reg: &mut Reg, _bus: &mut Bus) {}
}

// No operation
pub mod nop {
    use super::read;

    pub mod implied {
        pub fn t2(reg: &mut Reg, bus: &mut Bus) {
            reg.prefetch_into_m(bus);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod immediate {
        pub fn t2(reg: &mut Reg, bus: &mut Bus) {
            super::read::immediate::t2(reg, bus);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod zero_page {
        use super::read::zero_page::t2;

        pub fn t3(reg: &mut Reg, bus: &mut Bus) {
            super::read::zero_page::t3(reg, bus);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod zero_page_x {
        use super::read::zero_page_x::{t2, t3};

        pub fn t4(reg: &mut Reg, bus: &mut Bus) {
            super::read::zero_page_x::t4(reg, bus);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod absolute {
        use super::read::absolute::{t2, t3};

        pub fn t4(reg: &mut Reg, bus: &mut Bus) {
            super::read::absolute::t4(reg, bus);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod absolute_x {
        use super::read::absolute_x::{t2, t3, t4};

        pub fn t5(reg: &mut Reg, bus: &mut Bus) {
            super::read::absolute_x::t5(reg, bus);
            reg.set_last_cycle();
        }

        run!();
    }
}

// Load into A
pub mod lda {
    use super::read;

    pub mod immediate {
        pub fn t2(reg: &mut Reg, bus: &mut Bus) {
            super::read::immediate::t2(reg, bus);
            let operand = reg.read_m();
            reg.write_a(operand);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod zero_page {
        use super::read::zero_page::t2;

        pub fn t3(reg: &mut Reg, bus: &mut Bus) {
            let value = super::read::zero_page::t3(reg, bus);
            reg.write_a(value);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod zero_page_x {
        use super::read::zero_page_x::{t2, t3};

        pub fn t4(reg: &mut Reg, bus: &mut Bus) {
            let value = super::read::zero_page_x::t4(reg, bus);
            reg.write_a(value);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod absolute {
        use super::read::absolute::{t2, t3};

        pub fn t4(reg: &mut Reg, bus: &mut Bus) {
            let value = super::read::absolute::t4(reg, bus);
            reg.write_a(value);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod absolute_x {
        use super::read::absolute_x::{t2, t3};

        pub fn t4(reg: &mut Reg, bus: &mut Bus) {
            let value = super::read::absolute_x::t4(reg, bus);
            reg.write_a(value);
        }

        pub fn t5(reg: &mut Reg, bus: &mut Bus) {
            let value = super::read::absolute_x::t5(reg, bus);
            reg.write_a(value);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod absolute_y {
        use super::read::absolute_y::{t2, t3, t4};

        pub fn t5(reg: &mut Reg, bus: &mut Bus) {
            let value = super::read::absolute_y::t5(reg, bus);
            reg.write_a(value);
            reg.set_last_cycle();
        }

        run!();
    }

    pub mod indirect_x {
        use super::read::indirect_x::{t2, t3, t4, t5};

        pub fn t6(reg: &mut Reg, bus: &mut Bus) {
            let value = super::read::indirect_x::t6(reg, bus);
            reg.write_a(value);
        }

        run!();
    }
}