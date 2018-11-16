// Not all tests use everything declared here
#![allow(dead_code)]

extern crate nespera;

// Byte sizes
pub const UB2: u8 = 64;
pub const UB4: u8 = 128;
pub const UB6: u8 = UB4 + UB2;

// Most significant byte
pub fn msb(addr: u16) -> u8 { (addr >> 8) as u8 }

// Least significant byte
pub fn lsb(addr: u16) -> u8 { addr as u8 }

// Run an NES and check the result
#[macro_export]
macro_rules! run {
    // RAM and registers optional
    (
        opc: [ $( $opcode:expr ),* ];
        res: [ $( $reg:expr => $value:expr ),* ]
    ) => { run!(opc: [ $( $opcode ),* ]; reg: []; ram: []; res: [ $( $reg => $value ),* ] ) };

    // RAM optional
    (
        opc: [ $( $opcode:expr ),* ];
        reg: [ $( $initial_reg:ident => $initial_value:expr ),* ];
        res: [ $( $reg:expr => $value:expr ),* ]
    ) => { run!(
        opc: [ $( $opcode ),* ];
        reg: [ $( $initial_reg => $initial_value ),* ];
        ram: [];
        res: [ $( $reg => $value ),* ] )
    };

    // Registers optional
    (
        opc: [ $( $opcode:expr ),* ];
        ram: [ $( $addr:expr => $ram:expr ),* ];
        res: [ $( $reg:expr => $value:expr ),* ]
    ) => { run!(
        opc: [ $( $opcode ),* ];
        reg: [];
        ram: [ $( $addr => $ram ),* ];
        res: [ $( $reg => $value ),* ] )
    };

    // All parameters
    (
        opc: [ $( $opcode:expr ),* ];
        reg: [ $( $initial_reg:ident => $initial_value:expr ),* ];
        ram: [ $( $addr:expr => $ram:expr ),* ];
        res: [ $( $reg:expr => $value:expr ),* ]
    )
    => {{
        use std::collections::HashMap;

        use nespera::nes::Nes;
        use nespera::opc;

        let mut checks = HashMap::new();
        $( checks.insert($reg.to_string(), $value as u16); )*
        let get_value = |key, value| *checks.get(key).unwrap_or_else(|| &value) as u16;

        let mut nes = Nes::new();
        $( nes.write($opcode); )*

        $(
            let addr = $addr as u16;
            // Prevents from writing twice at the same address which can hinder testing
            if nes.peek_at(addr) != 0 { panic!("Trying to write twice in 0x{:x?}", addr); }
            nes.put_at(addr, $ram as u8);
        )*

        $( nes.cpu.$initial_reg = ($initial_value as u8).into(); )*

        let control = nes.clone();

        nes.cpu.reset_pc();
        while nes.peek() != 0 { nes.step(); }

        assert_eq!(nes.cpu.get_a(), get_value("a", control.cpu.get_a() as u16) as u8, "\n in register a");
        assert_eq!(nes.cpu.get_x(), get_value("x", control.cpu.get_x() as u16) as u8, "\n in register x");
        assert_eq!(nes.cpu.get_y(), get_value("y", control.cpu.get_y() as u16) as u8, "\n in register y");

        assert_eq!(nes.cpu.get_pc(), get_value("pc", control.cpu.get_pc() as u16) as u16, "\n in register pc");
        assert_eq!(nes.cpu.get_sp(), get_value("sp", control.cpu.get_sp() as u16) as u8, "\n in register sp");

        assert_eq!(nes.cpu.get_c(), get_value("c", control.cpu.get_c() as u16) != 0, "\n in carry flag");
        assert_eq!(nes.cpu.get_z(), get_value("z", control.cpu.get_z() as u16) != 0, "\n in zero flag");
        assert_eq!(nes.cpu.get_i(), get_value("i", control.cpu.get_i() as u16) != 0, "\n in interrupt disable flag");
        assert_eq!(nes.cpu.get_d(), get_value("d", control.cpu.get_d() as u16) != 0, "\n in decimal mode");
        assert_eq!(nes.cpu.get_b(), get_value("b", control.cpu.get_b() as u16) != 0, "\n in break command");
        assert_eq!(nes.cpu.get_u(), get_value("u", control.cpu.get_u() as u16) != 0, "\n in unused");
        assert_eq!(nes.cpu.get_o(), get_value("o", control.cpu.get_o() as u16) != 0, "\n in overflow");
        assert_eq!(nes.cpu.get_n(), get_value("n", control.cpu.get_n() as u16) != 0, "\n in negative");

        for (i, (&ea, &eb)) in nes.ram.0.iter().zip(control.ram.0.iter()).enumerate() {
            let ref_eb = &(eb as u16);
            let value = *checks.get(&i.to_string()[..]).unwrap_or_else(|| ref_eb) as u8;
            assert_eq!(ea, value, "\nat address {}", i);
        }

        nes
    }}
}
