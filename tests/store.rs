extern crate nespera;

mod cpu;

use cpu::*;

// Tests for zero page
macro_rules! zero_page {
    ( $opcode:ident, $register:ident ) => {
        mod zero_page {
            use super::*;

            #[test]
            fn regular() {
                run!(opc: [opc::$opcode::ZeroPage, 8];
                    reg: [$register => 127];
                    res: [8 => 127]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::ZeroPage, 255,
                        opc::$opcode::ZeroPage, 7,
                        opc::$opcode::ZeroPage, 12
                    ];
                    reg: [$register => 213];
                    res: [255 => 213, 7 => 213, 12 => 213]);
            }
        }
    };
}

// Tests for zero page with register
macro_rules! zero_page_reg {
    ( $module:ident, $opcode:ident, $mode:ident, $zero_reg:ident, $stored_reg:ident ) => {
        mod $module {
            use super::*;

            #[test]
            fn regular() {
                run!(opc: [opc::$opcode::$mode, 8];
                    reg: [$stored_reg => 28, $zero_reg => 12];
                    res: [20 => 28]);
            }

            #[test]
            fn overflow() {
                run!(opc: [opc::$opcode::$mode, UB6 + 8];
                    reg: [$stored_reg => 28, $zero_reg => UB6 + 12];
                    res: [UB4 + 20 => 28]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::$mode, 255,
                        opc::$opcode::$mode, 52,
                        opc::$opcode::$mode, 8
                    ];
                    reg: [$stored_reg => 144, $zero_reg => 10];
                    res: [9 => 144, 62 => 144, 18 => 144]);
            }
        }
    };
}

// Tests for absolute
macro_rules! absolute {
    ( $opcode:ident, $register:ident ) => {
        mod absolute {
            use super::*;

            #[test]
            fn one_byte() {
                run!(opc: [opc::$opcode::Absolute, lsb(8), msb(8)];
                    reg: [$register => 144];
                    res: [8 => 144]);
            }

            #[test]
            fn two_bytes() {
                run!(opc: [opc::$opcode::Absolute, lsb(750), msb(750)];
                    reg: [$register => 144];
                    res: [750 => 144]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::Absolute, lsb(32), msb(32),
                        opc::$opcode::Absolute, lsb(15), msb(15),
                        opc::$opcode::Absolute, lsb(750), msb(750)
                    ];
                    reg: [$register => 144];
                    res: [32 => 144, 15 => 144, 750 => 144]);
            }
        }
    };
}

// Tests for absolutes with register
macro_rules! absolute_reg {
    ( $module:ident, $mode:ident, $reg:ident ) => {
        mod $module {
            use super::*;

            #[test]
            fn one_byte() {
                run!(opc: [opc::Sta::$mode, lsb(8), msb(8)];
                    reg: [a => 144, $reg => 10];
                    res: [18 => 144]);
            }

            #[test]
            fn two_bytes() {
                run!(opc: [opc::Sta::$mode, lsb(750), msb(750)];
                    reg: [a => 144, $reg => 10];
                    res: [760 => 144]);
            }

            #[test]
            fn overflow() {
                run!(opc: [opc::Sta::$mode, lsb(0xfffe), msb(0xfffe)];
                     reg: [a => 144, $reg => 14];
                     res: [12 => 144]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::Sta::$mode, lsb(32), msb(32),
                        opc::Sta::$mode, lsb(15), msb(15),
                        opc::Sta::$mode, lsb(750), msb(750)
                    ];
                    reg: [a => 144, $reg => 10];
                    res: [42 => 144, 25 => 144, 760 => 144]);
            }
        }
    };
}

mod sta {
    use super::*;

    zero_page!(Sta, a);

    zero_page_reg!(zero_page_x, Sta, ZeroPageX, x, a);

    absolute!(Sta, a);

    absolute_reg!(absolute_x, AbsoluteX, x);

    absolute_reg!(absolute_y, AbsoluteY, y);

    mod indirect_x {
        use super::*;

        #[test]
        fn regular() {
            run!(opc: [opc::Sta::IndirectX, 15];
                reg: [a => 244, x => 2];
                ram: [17 => 0x07, 18 => 0x10, 0x0710 => 52];
                res: [0x0710 => 244]);
        }

        #[test]
        fn overflow() {
            run!(opc: [opc::Sta::IndirectX, UB6 + 15];
                reg: [a => 244, x => UB6 + 2];
                ram: [UB4 + 17 => 0x07, UB4 + 18 => 0x10];
                res: [0x0710 => 244]);
        }

        #[test]
        fn multiple_opcode() {
            run!(opc: [
                    opc::Sta::IndirectX, 20,
                    opc::Sta::IndirectX, 112
                ];
                reg: [a => 244, x => 2];
                ram: [
                    22 => 0x06, 23 => 0x09,
                    114 => 0x07, 115 => 0x10];
                res: [0x0609 => 244, 0x0710 => 244]);
        }
    }

    mod indirect_y {
        use super::*;

        #[test]
        fn regular() {
            run!(opc: [opc::Sta::IndirectY, 0x2a];
                reg: [a => 244, y => 0x03];
                ram: [0x2a => 0x07, 0x2b => 0x35];
                res: [0x0738 => 244]);
        }

        #[test]
        fn overflow() {
            run!(opc: [opc::Sta::IndirectY, 0x2a];
                reg: [a => 244, y => 0x28];
                ram: [0x2a => 0xff, 0x2b => 0xfe];
                res: [0x26 => 244]);
        }

        #[test]
        fn multiple_opcode() {
            run!(opc: [
                    opc::Sta::IndirectY, 0x2a,
                    opc::Sta::IndirectY, 0x35
                ];
                reg: [a => 244, y => 0x50];
                ram: [
                    0x2a => 0x07, 0x2b => 0x35,
                    0x35 => 0xff, 0x36 => 0xfe];
                res: [0x0785 => 244, 0x4e => 244]);
        }
    }
}

mod stx {
    use super::*;

    zero_page!(Stx, x);

    zero_page_reg!(zero_page_y, Stx, ZeroPageY, y, x);

    absolute!(Stx, x);
}

mod sty {
    use super::*;

    zero_page!(Sty, y);

    zero_page_reg!(zero_page_x, Sty, ZeroPageX, x, y);

    absolute!(Sty, y);
}