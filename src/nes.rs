use cpu::Cpu;
use opc;
use flags::Flags;
use std::num::Wrapping;

// RAM
const RAM_CAPACITY: usize = 0x0800;

#[derive(Copy, Clone)]
pub struct Ram(pub [u8; RAM_CAPACITY]);

// NES
#[derive(Copy, Clone)]
pub struct Nes {
    pub cpu: Cpu,
    pub ram: Ram,
}

impl Nes {
    // Constructor
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            ram: Ram([0; RAM_CAPACITY]),
        }
    }

    // Getters for registers
    pub fn get_a(&self) -> u8 { self.cpu.a }
    pub fn get_x(&self) -> u8 { self.cpu.x }
    pub fn get_y(&self) -> u8 { self.cpu.y }
    pub fn get_pc(&self) -> u16 { self.cpu.pc }
    pub fn get_sp(&self) -> u8 { self.cpu.sp }

    // Getters for flags
    pub fn get_c(&self) -> bool { self.cpu.p.intersects(Flags::Carry) }
    pub fn get_z(&self) -> bool { self.cpu.p.intersects(Flags::Zero) }
    pub fn get_i(&self) -> bool { self.cpu.p.intersects(Flags::InterruptDisable) }
    pub fn get_d(&self) -> bool { self.cpu.p.intersects(Flags::DecimalMode) }
    pub fn get_b(&self) -> bool { self.cpu.p.intersects(Flags::BreakCommand) }
    pub fn get_u(&self) -> bool { self.cpu.p.intersects(Flags::Unused) }
    pub fn get_o(&self) -> bool { self.cpu.p.intersects(Flags::Overflow) }
    pub fn get_n(&self) -> bool { self.cpu.p.intersects(Flags::Negative) }

    // Read the value in RAM pointed by the address
    pub fn peek_at(&self, addr: u16) -> u8 {
        self.ram.0[addr as usize]
    }

    // Read the value in RAM pointed by PC without changing it
    pub fn peek(&self) -> u8 {
        self.peek_at(self.cpu.get_pc())
    }

    // Write the value to RAM pointed by the address
    pub fn put_at(&mut self, addr: u16, value: u8) {
        self.ram.0[addr as usize] = value;
    }

    // Write the value to RAM pointed by PC without changing it
    pub fn put(&mut self, value: u8) {
        let addr = self.cpu.get_pc();
        self.put_at(addr, value);
    }

    // Read the value in RAM pointed by PC and advances it
    pub fn fetch(&mut self) -> u8 {
        let data = self.peek();
        self.cpu.inc_pc();
        data
    }

    // Write the value to RAM pointed by PC and advances it
    pub fn write(&mut self, value: u8) {
        self.put(value);
        self.cpu.inc_pc();
    }

    // Value getters
    fn immediate(&mut self) -> u8 { self.fetch() }

    fn zero_page(&mut self) -> u8 {
        let addr = self.fetch();
        self.peek_at(addr.into())
    }

    fn zero_page_x(&mut self) -> u8 {
        let addr = Wrapping(self.fetch()) + Wrapping(self.cpu.x);
        self.peek_at(addr.0.into())
    }

    fn zero_page_y(&mut self) -> u8 {
        let addr = Wrapping(self.fetch()) + Wrapping(self.cpu.y);
        self.peek_at(addr.0.into())
    }

    fn absolute(&mut self) -> u8 {
        let high = (self.fetch() as u16) << 8;
        let low = self.fetch() as u16;
        self.peek_at(high + low)
    }

    fn absolute_x(&mut self) -> u8 {
        let high = (self.fetch() as u16) << 8;
        let low = self.fetch() as u16;
        let addr = Wrapping(high + low) + Wrapping(self.cpu.x as u16);
        self.peek_at(addr.0)
    }

    fn absolute_y(&mut self) -> u8 {
        let high = (self.fetch() as u16) << 8;
        let low = self.fetch() as u16;
        let addr = Wrapping(high + low) + Wrapping(self.cpu.y as u16);
        self.peek_at(addr.0)
    }

    fn indirect_x(&mut self) -> u8 {
        let addr = Wrapping(self.fetch()) + Wrapping(self.cpu.x);
        let high = (self.peek_at(addr.0.into()) as u16) << 8;
        let low = self.peek_at((addr + Wrapping(1)).0.into()) as u16;
        self.peek_at(high + low)
    }

    fn indirect_y(&mut self) -> u8 {
        let addr = Wrapping(self.fetch());
        let high = (self.peek_at(addr.0.into()) as u16) << 8;
        let low = self.peek_at((addr + Wrapping(1)).0.into()) as u16;
        self.peek_at((Wrapping(high + low) + Wrapping(self.cpu.y as u16)).0)
    }

    // Executes one step of the CPU
    pub fn step(&mut self) {
        // Macro for registers
        macro_rules! set_reg {
            ( $setter:ident, $getter:ident ) => {{
                let value = self.$getter();
                self.cpu.$setter(value);
            }};
        }

        // Macro for memory

        let opcode = self.fetch();

        match opcode {
            // Load into A
            opc::Lda::Immediate => set_reg!(set_a, immediate),
            opc::Lda::ZeroPage => set_reg!(set_a, zero_page),
            opc::Lda::ZeroPageX => set_reg!(set_a, zero_page_x),
            opc::Lda::Absolute => set_reg!(set_a, absolute),
            opc::Lda::AbsoluteX => set_reg!(set_a, absolute_x),
            opc::Lda::AbsoluteY => set_reg!(set_a, absolute_y),
            opc::Lda::IndirectX => set_reg!(set_a, indirect_x),
            opc::Lda::IndirectY => set_reg!(set_a, indirect_y),

            // Load into X
            opc::Ldx::Immediate => set_reg!(set_x, immediate),
            opc::Ldx::ZeroPage => set_reg!(set_x, zero_page),
            opc::Ldx::ZeroPageY => set_reg!(set_x, zero_page_y),
            opc::Ldx::Absolute => set_reg!(set_x, absolute),
            opc::Ldx::AbsoluteY => set_reg!(set_x, absolute_y),

            // Load into Y
            opc::Ldy::Immediate => set_reg!(set_y, immediate),
            opc::Ldy::ZeroPage => set_reg!(set_y, zero_page),
            opc::Ldy::ZeroPageX => set_reg!(set_y, zero_page_x),
            opc::Ldy::Absolute => set_reg!(set_y, absolute),
            opc::Ldy::AbsoluteX => set_reg!(set_y, absolute_x),

            // Not implemented
            _ => panic!("Opcode not implemented")
        }
    }
}
