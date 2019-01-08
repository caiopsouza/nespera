use std::io::Write;
use std::sync::RwLock;

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

pub use crate::cpu::log::logging::*;
use crate::cpu::reg::Reg;

// Starts logging after the specified amount of logs has passed.
// Tracing is very verbose so you might need to limit how much is logged in order to speed up execution.
pub fn setup(level: LevelFilter, start_after: usize) {
    let counter = RwLock::new(0);
    Builder::new()
        .format(move |buf, record| {
            let mut counter = counter.write().unwrap();
            *counter += 1;
            if *counter < start_after { return Result::Ok(()); }
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S.%f"),
                     record.level(),
                     record.args()
            )
        })
        .filter(None, level)
        .init();
}


#[derive(Debug, Copy, Clone)]
pub enum AddrMode {
    Unknown,
    Implied,
    Accumulator,
    Immediate(u8),
    ZeroPage(u8, u8),
    ZeroPageX(u8, u8, u8),
    ZeroPageY(u8, u8, u8),
    Relative(u8, u16),
    Absolute(u16, u8),
    AbsoluteX(u16, u16, u8),
    AbsoluteY(u16, u16, u8),
    Direct(u16),
    Indirect(u16, u16),
    IndirectX(u8, u8, u16, u8),
    IndirectY(u8, u16, u16, u8),
}

// Dummy implementation. Used when not logging.
#[cfg(not(debug_assertions))]
pub mod logging {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct Log;

    impl Log {
        pub fn new() -> Self { Self }

        pub fn get(&self) -> String { "".to_owned() }

        pub fn set_skip(&self, _: bool) {}
        pub fn set_reg(&self, _: Reg) {}
        pub fn set_unofficial(&self, _: bool) {}
        pub fn set_mnemonic(&self, _: &'static str) {}
        pub fn set_mode(&self, _: AddrMode) {}
        pub fn set_dot(&self, _: u32) {}
        pub fn set_scanline(&self, _: i32) {}
    }

    impl Default for Log { fn default() -> Self { Self::new() } }
}

// Actual implementation
#[cfg(debug_assertions)]
pub mod logging {
    use crate::bus::Bus;
    use crate::utils::bits;

    use super::*;

    fn f_u8(data: u8) -> String { format!("{:02X}", data) }

    fn f_u16(data: u16) -> String { format!("{:04X}", data) }

    fn f_mem(addr: u16, data: u8) -> String {
        // Mesen reports PPU and APU reads on the logs as zero for some reason.
        let data = match addr {
            0x2000...0x3fff => 0,
            0x4000...0x401f => 0,
            _ => data,
        };
        format!("${} = ${}", f_u16(addr), f_u8(data))
    }

    #[derive(Debug, Clone)]
    pub struct Log {
        skip: bool,
        reg: Reg,
        unofficial: bool,
        mnemonic: &'static str,
        mode: AddrMode,
        dot: u32,
        scanline: i32,
        frame: u32,
        clock: u32,
    }

    impl AddrMode {
        fn operands(self) -> String {
            match self {
                AddrMode::Unknown => "?? ??".to_owned(),

                | AddrMode::Implied
                | AddrMode::Accumulator
                => "".to_owned(),

                | AddrMode::Immediate(op0)
                | AddrMode::ZeroPage(op0, _)
                | AddrMode::ZeroPageX(op0, _, _)
                | AddrMode::ZeroPageY(op0, _, _)
                | AddrMode::Relative(op0, _)
                | AddrMode::IndirectX(op0, _, _, _)
                | AddrMode::IndirectY(op0, _, _, _)
                => format!("${}", f_u8(op0)),

                | AddrMode::Absolute(operand, _)
                | AddrMode::AbsoluteX(operand, _, _)
                | AddrMode::AbsoluteY(operand, _, _)
                | AddrMode::Direct(operand)
                | AddrMode::Indirect(operand, _)
                => format!("${} ${}", f_u8(bits::low(operand)), f_u8(bits::high(operand))),
            }
        }

        fn operation(self, bus: &Bus) -> String {
            match self {
                AddrMode::Unknown => "???".to_owned(),
                AddrMode::Implied => "".to_owned(),
                AddrMode::Accumulator => "A".to_owned(),
                AddrMode::Immediate(data) => format!("#${}", f_u8(data)),
                AddrMode::ZeroPage(addr, data) => format!("${} = ${}", f_u8(addr), f_u8(data)),
                AddrMode::ZeroPageX(operand, addr, data) => format!("${},X @ ${} = ${}", f_u8(operand), f_u8(addr), f_u8(data)),
                AddrMode::ZeroPageY(operand, addr, data) => format!("${},Y @ ${} = ${}", f_u8(operand), f_u8(addr), f_u8(data)),
                AddrMode::Absolute(addr, data) => f_mem(addr, data),
                AddrMode::AbsoluteX(operand, addr, data) => format!("${},X @ {}", f_u16(operand), f_mem(addr, data)),
                AddrMode::AbsoluteY(operand, addr, data) => format!("${},Y @ {}", f_u16(operand), f_mem(addr, data)),
                AddrMode::Relative(_, addr) | AddrMode::Direct(addr) => f_mem(addr, bus.peek_cpu(addr)),
                AddrMode::Indirect(operand, data) => format!("(${}) = {}", f_u16(operand), f_u16(data)),
                AddrMode::IndirectX(operand, index, addr, data) => format!("(${},X) @ ${} = {}", f_u8(operand), f_u8(index), f_mem(addr, data)),
                AddrMode::IndirectY(operand, _, addr, data) => format!("(${}),Y @ {}", f_u8(operand), f_mem(addr, data)),
            }
        }
    }

    impl Log {
        pub fn new() -> Self {
            Self {
                skip: true,
                reg: Reg::new(),
                unofficial: false,
                mnemonic: "???",
                mode: AddrMode::Unknown,
                dot: 0,
                scanline: 0,
                frame: 0,
                clock: 0,
            }
        }

        pub fn set_skip(&mut self, skip: bool) { self.skip = skip }
        pub fn set_reg(&mut self, reg: Reg) { self.reg = reg }
        pub fn set_unofficial(&mut self, unofficial: bool) { self.unofficial = unofficial }
        pub fn set_mnemonic(&mut self, mnemonic: &'static str) { self.mnemonic = mnemonic }
        pub fn set_dot(&mut self, dot: u32) { self.dot = dot }
        pub fn set_scanline(&mut self, scanline: i32) { self.scanline = scanline }
        pub fn set_frame(&mut self, frame: u32) { self.frame = frame }
        pub fn set_clock(&mut self, clock: u32) { self.clock = clock }

        fn is_logging(&self) -> bool {
            /*log::STATIC_MAX_LEVEL >= log::LevelFilter::Trace
                && log::max_level() >= log::LevelFilter::Trace
                && !self.skip*/ true
        }

        pub fn set_mode(&mut self, mode: AddrMode) {
            if self.is_logging() { self.mode = mode; }
        }

        pub fn get(&self, bus: &Bus) -> String {
            if !self.is_logging() { return "".to_owned(); }

            format!("{} ${} {:7} {} {:<33} A:{} X:{} Y:{} P:{} SP:{} CYC:{:<3} SL:{:<3} FC:{} CPU Cycle:{}",
                    f_u16(self.reg.get_pc()),
                    f_u8(self.reg.get_current_instr()),
                    self.mode.operands(),
                    self.mnemonic,
                    self.mode.operation(bus),
                    f_u8(self.reg.get_a()),
                    f_u8(self.reg.get_x()),
                    f_u8(self.reg.get_y()),
                    f_u8(self.reg.get_p().into()),
                    f_u8(self.reg.get_s()),
                    self.dot,
                    self.scanline,
                    self.frame,
                    self.clock
            )
        }
    }

    impl Default for Log { fn default() -> Self { Self::new() } }
}
