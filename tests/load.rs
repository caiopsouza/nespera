extern crate nespera;

mod cpu;

use cpu::*;

// Tests for immediate
macro_rules! immediate {
    ( $opcode:ident, $register:expr ) => {
        mod immediate {
            use super::*;

            #[test]
            fn positive() {
                run!(opc: [opc::$opcode::Immediate, 127];
                    res: [$register => 127, "n" => false, "z" => false]);
            }

            #[test]
            fn boundary() {
                run!(opc: [opc::$opcode::Immediate, 255];
                    res: [$register => 255, "n" => true, "z" => false]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::Immediate, 255,
                        opc::$opcode::Immediate, 0,
                        opc::$opcode::Immediate, 1
                    ];
                    res: [$register => 1, "n" => false, "z" => false]);
            }
        }
    };
}

// Tests for zero page
macro_rules! zero_page {
    ( $opcode:ident, $register:expr ) => {
        mod zero_page {
            use super::*;

            #[test]
            fn regular() {
                run!(opc: [opc::$opcode::ZeroPage, 8];
                    ram: [8 => 127];
                    res: [$register => 127, "n" => false, "z" => false]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::ZeroPage, 255,
                        opc::$opcode::ZeroPage, 0,
                        opc::$opcode::ZeroPage, 8
                    ];
                    ram: [8 => 213];
                    res: [$register => 213, "n" => true, "z" => false]);
            }
        }
    };
}

// Tests for zero page with register
macro_rules! zero_page_reg {
    ( $module:ident, $opcode:ident, $mode:ident, $zero_reg:ident, $res_reg:expr ) => {
        mod $module {
            use super::*;

            #[test]
            fn regular() {
                run!(opc: [opc::$opcode::$mode, 8];
                    reg: [$zero_reg => 7];
                    ram: [15 => 127];
                    res: [$res_reg => 127, "n" => false, "z" => false]);
            }

            #[test]
            fn overflow() {
                run!(opc: [opc::$opcode::$mode, UB6 + 1];
                    reg: [$zero_reg => UB6 + 2];
                    ram: [UB4 + 3 => 255];
                    res: [$res_reg => 255, "n" => true, "z" => false]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::$mode, 255,
                        opc::$opcode::$mode, 0,
                        opc::$opcode::$mode, 1
                    ];
                    reg: [$zero_reg => 7];
                    ram: [8 => 255];
                    res: [$res_reg => 255, "n" => true, "z" => false]);
            }
        }
    };
}

// Tests for absolute
macro_rules! absolute {
    ( $opcode:ident, $register:expr ) => {
        mod absolute {
            use super::*;

            #[test]
            fn one_byte() {
                run!(opc: [opc::$opcode::Absolute, lsb(8), msb(8)];
                     ram: [8 => 127];
                     res: [$register => 127, "n" => false, "z" => false]);
            }

            #[test]
            fn two_bytes() {
                run!(opc: [opc::$opcode::Absolute, lsb(750), msb(750)];
                     ram: [750 => 212];
                     res: [$register => 212, "n" => true, "z" => false]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::Absolute, lsb(1), msb(1),
                        opc::$opcode::Absolute, lsb(0), msb(0),
                        opc::$opcode::Absolute, lsb(750), msb(750)
                    ];
                    ram: [750 => 212];
                    res: [$register => 212, "n" => true, "z" => false]);
            }
        }
    };
}

// Tests for absolutes with register
macro_rules! absolute_reg {
    ( $module:ident, $opcode:ident, $mode:ident, $absolute_reg:ident, $res_reg:expr ) => {
        mod $module {
            use super::*;

            #[test]
            fn one_byte() {
                run!(opc: [opc::$opcode::$mode, lsb(8), msb(8)];
                     reg: [$absolute_reg => 7];
                     ram: [15 => 127];
                     res: [$res_reg => 127, "n" => false, "z" => false]);
            }

            #[test]
            fn two_bytes() {
                run!(opc: [opc::$opcode::$mode, lsb(750), msb(750)];
                     reg: [$absolute_reg => 7];
                     ram: [757 => 212];
                     res: [$res_reg => 212, "n" => true, "z" => false]);
            }

            #[test]
            fn overflow() {
                run!(opc: [opc::$opcode::$mode, lsb(0xfffe), msb(0xfffe)];
                     reg: [$absolute_reg => 7];
                     ram: [5 => 212];
                     res: [$res_reg => 212, "n" => true, "z" => false]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::$mode, lsb(1), msb(1),
                        opc::$opcode::$mode, lsb(0), msb(0),
                        opc::$opcode::$mode, lsb(750), msb(750)
                    ];
                    reg: [$absolute_reg => 7];
                    ram: [757 => 212];
                    res: [$res_reg => 212, "n" => true, "z" => false]);
            }
        }
    };
}

// Tests for indirect X
macro_rules! indirect_x {
    ( $opcode:ident, $register:expr ) => {
        mod indirect_x {
            use super::*;

            #[test]
            fn regular() {
                run!(opc: [opc::$opcode::IndirectX, 15];
                    reg: [x => 2];
                    ram: [17 => 0x07, 18 => 0x10, 0x0710 => 20];
                    res: [$register => 20, "n" => false, "z" => false]);
            }

            #[test]
            fn overflow() {
                run!(opc: [opc::$opcode::IndirectX, UB6 + 15];
                    reg: [x => UB6 + 2];
                    ram: [UB4 + 17 => 0x07, UB4 + 18 => 0x10, 0x0710 => 20];
                    res: [$register => 20, "n" => false, "z" => false]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::IndirectX, 5,
                        opc::$opcode::IndirectX, 15
                    ];
                    reg: [x => 2];
                    ram: [
                        7 => 0x06, 8 => 0x09, 0x0609 => 78,
                        17 => 0x07, 18 => 0x10, 0x0710 => 131];
                    res: [$register => 131, "n" => true, "z" => false]);
            }
        }
    };
}

// Tests for indirect Y
macro_rules! indirect_y {
    ( $opcode:ident, $register:expr ) => {
        mod indirect_y {
            use super::*;

            #[test]
            fn regular() {
                run!(opc: [opc::$opcode::IndirectY, 0x2a];
                    reg: [y => 0x03];
                    ram: [0x2a => 0x07, 0x2b => 0x35, 0x0738 => 0x2f];
                    res: [$register => 0x2f, "n" => false, "z" => false]);
            }

            #[test]
            fn overflow() {
                run!(opc: [opc::$opcode::IndirectY, 0x35];
                    reg: [y => 0x10];
                    ram: [0x35 => 0xff, 0x36 => 0xfe, 0x0e => 0x2f];
                    res: [$register => 0x2f, "n" => false, "z" => false]);
            }

            #[test]
            fn multiple_opcode() {
                run!(opc: [
                        opc::$opcode::IndirectY, 0x2a,
                        opc::$opcode::IndirectY, 0x35
                    ];
                    reg: [y => 0x10];
                    ram: [
                        0x2a => 0x07, 0x2b => 0x35, 0x0745 => 0x2a,
                        0x35 => 0xff, 0x36 => 0xfe, 0x0e => 0x2f];
                    res: [$register => 0x2f, "n" => false, "z" => false]);
            }
        }
    };
}

mod lda {
    use super::*;

    immediate!(Lda, "a");

    zero_page!(Lda, "a");

    zero_page_reg!(zero_page_x, Lda, ZeroPageX, x, "a");

    absolute!(Lda, "a");

    absolute_reg!(absolute_x, Lda, AbsoluteX, x, "a");

    absolute_reg!(absolute_y, Lda, AbsoluteY, y, "a");

    indirect_x!(Lda, "a");

    indirect_y!(Lda, "a");
}

mod ldx {
    use super::*;

    immediate!(Ldx, "x");

    zero_page!(Ldx, "x");

    zero_page_reg!(zero_page_y, Ldx, ZeroPageY, y, "x");

    absolute!(Ldx, "x");

    absolute_reg!(absolute_y, Ldx, AbsoluteY, y, "x");
}

mod ldy {
    use super::*;

    immediate!(Ldy, "y");

    zero_page!(Ldy, "y");

    zero_page_reg!(zero_page_x, Ldy, ZeroPageX, x, "y");

    absolute!(Ldy, "y");

    absolute_reg!(absolute_x, Ldy, AbsoluteX, x, "y");
}