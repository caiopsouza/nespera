// Addressing mode.
// Different modes of fetching data from memory.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AddrMode {
    // Address mode is implied by the instruction
    Implicit,

    // Operates on the accumulator
    Accumulator,

    // Uses the next byte in memory
    Immediate,

    // Reads from mem as specified by the immediate address
    ZeroPage,

    // Similar to ZeroPage but adds Z
    ZeroPageX,

    // Similar to ZeroPage but adds Y
    ZeroPageY,

    // Next byte is used as offset
    Relative,

    // Next two bytes are used as absolute value
    Absolute,

    // Similar to Absolute but adds X
    AbsoluteX,

    // Similar to Absolute but adds Y
    AbsoluteY,

    // Indirect mode has the address of memory
    Indirect,

    // Similar to Indirect, but adds X to fetch the value
    IndirectX,

    // Similar to Indirect, but adds Y after fetching the value
    IndirectY,
}

// Operations to execute
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Operation {
    // Not implemented option
    None,

    // No-op. Does nothing (usually)
    Nop,

    // Kill the processor
    Stop,

    // Logical Or
    Or,
}

// Opcode
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Opcode {
    name: &'static str,
    oper: Operation,
    mode: AddrMode,
}

pub const NONE: Opcode = Opcode { name: "___", oper: Operation::None, mode: AddrMode::Implicit };

pub const NOP: Opcode = Opcode { name: "NOP", oper: Operation::Nop, mode: AddrMode::Implicit };
pub const NOP_IMM: Opcode = Opcode { name: "NOP", oper: Operation::Nop, mode: AddrMode::Immediate };


pub const STP: Opcode = Opcode { name: "STP", oper: Operation::Stop, mode: AddrMode::Implicit };

pub const OPCODES: [Opcode; 256] = [
    /*00*/ NONE,
    /*01*/ NONE,
    /*02*/ STP,
    /*03*/ NONE,
    /*04*/ NONE,
    /*05*/ NONE,
    /*06*/ NONE,
    /*07*/ NONE,
    /*08*/ NONE,
    /*09*/ NONE,
    /*0a*/ NONE,
    /*0b*/ NONE,
    /*0c*/ NONE,
    /*0d*/ NONE,
    /*0e*/ NONE,
    /*0f*/ NONE,
    /**/
    /*10*/ NONE,
    /*11*/ NONE,
    /*12*/ STP,
    /*13*/ NONE,
    /*14*/ NONE,
    /*15*/ NONE,
    /*16*/ NONE,
    /*17*/ NONE,
    /*18*/ NONE,
    /*19*/ NONE,
    /*1a*/ NONE,
    /*1b*/ NONE,
    /*1c*/ NONE,
    /*1d*/ NONE,
    /*1e*/ NONE,
    /*1f*/ NONE,
    /**/
    /*20*/ NONE,
    /*21*/ NONE,
    /*22*/ STP,
    /*23*/ NONE,
    /*24*/ NONE,
    /*25*/ NONE,
    /*26*/ NONE,
    /*27*/ NONE,
    /*28*/ NONE,
    /*29*/ NONE,
    /*2a*/ NONE,
    /*2b*/ NONE,
    /*2c*/ NONE,
    /*2d*/ NONE,
    /*2e*/ NONE,
    /*2f*/ NONE,
    /**/
    /*30*/ NONE,
    /*31*/ NONE,
    /*32*/ STP,
    /*33*/ NONE,
    /*34*/ NONE,
    /*35*/ NONE,
    /*36*/ NONE,
    /*37*/ NONE,
    /*38*/ NONE,
    /*39*/ NONE,
    /*3a*/ NONE,
    /*3b*/ NONE,
    /*3c*/ NONE,
    /*3d*/ NONE,
    /*3e*/ NONE,
    /*3f*/ NONE,
    /**/
    /*40*/ NONE,
    /*41*/ NONE,
    /*42*/ STP,
    /*43*/ NONE,
    /*44*/ NONE,
    /*45*/ NONE,
    /*46*/ NONE,
    /*47*/ NONE,
    /*48*/ NONE,
    /*49*/ NONE,
    /*4a*/ NONE,
    /*4b*/ NONE,
    /*4c*/ NONE,
    /*4d*/ NONE,
    /*4e*/ NONE,
    /*4f*/ NONE,
    /**/
    /*50*/ NONE,
    /*51*/ NONE,
    /*52*/ STP,
    /*53*/ NONE,
    /*54*/ NONE,
    /*55*/ NONE,
    /*56*/ NONE,
    /*57*/ NONE,
    /*58*/ NONE,
    /*59*/ NONE,
    /*5a*/ NONE,
    /*5b*/ NONE,
    /*5c*/ NONE,
    /*5d*/ NONE,
    /*5e*/ NONE,
    /*5f*/ NONE,
    /**/
    /*60*/ NONE,
    /*61*/ NONE,
    /*62*/ STP,
    /*63*/ NONE,
    /*64*/ NONE,
    /*65*/ NONE,
    /*66*/ NONE,
    /*67*/ NONE,
    /*68*/ NONE,
    /*69*/ NONE,
    /*6a*/ NONE,
    /*6b*/ NONE,
    /*6c*/ NONE,
    /*6d*/ NONE,
    /*6e*/ NONE,
    /*6f*/ NONE,
    /**/
    /*70*/ NONE,
    /*71*/ NONE,
    /*72*/ STP,
    /*73*/ NONE,
    /*74*/ NONE,
    /*75*/ NONE,
    /*76*/ NONE,
    /*77*/ NONE,
    /*78*/ NONE,
    /*79*/ NONE,
    /*7a*/ NONE,
    /*7b*/ NONE,
    /*7c*/ NONE,
    /*7d*/ NONE,
    /*7e*/ NONE,
    /*7f*/ NONE,
    /**/
    /*80*/ NONE,
    /*81*/ NONE,
    /*82*/ NONE,
    /*83*/ NONE,
    /*84*/ NONE,
    /*85*/ NONE,
    /*86*/ NONE,
    /*87*/ NONE,
    /*88*/ NONE,
    /*89*/ NONE,
    /*8a*/ NONE,
    /*8b*/ NONE,
    /*8c*/ NONE,
    /*8d*/ NONE,
    /*8e*/ NONE,
    /*8f*/ NONE,
    /**/
    /*90*/ NONE,
    /*91*/ NONE,
    /*92*/ STP,
    /*93*/ NONE,
    /*94*/ NONE,
    /*95*/ NONE,
    /*96*/ NONE,
    /*97*/ NONE,
    /*98*/ NONE,
    /*99*/ NONE,
    /*9a*/ NONE,
    /*9b*/ NONE,
    /*9c*/ NONE,
    /*9d*/ NONE,
    /*9e*/ NONE,
    /*9f*/ NONE,
    /**/
    /*a0*/ NONE,
    /*a1*/ NONE,
    /*a2*/ NONE,
    /*a3*/ NONE,
    /*a4*/ NONE,
    /*a5*/ NONE,
    /*a6*/ NONE,
    /*a7*/ NONE,
    /*a8*/ NONE,
    /*a9*/ NONE,
    /*aa*/ NONE,
    /*ab*/ NONE,
    /*ac*/ NONE,
    /*ad*/ NONE,
    /*ae*/ NONE,
    /*af*/ NONE,
    /**/
    /*b0*/ NONE,
    /*b1*/ NONE,
    /*b2*/ STP,
    /*b3*/ NONE,
    /*b4*/ NONE,
    /*b5*/ NONE,
    /*b6*/ NONE,
    /*b7*/ NONE,
    /*b8*/ NONE,
    /*b9*/ NONE,
    /*ba*/ NONE,
    /*bb*/ NONE,
    /*bc*/ NONE,
    /*bd*/ NONE,
    /*be*/ NONE,
    /*bf*/ NONE,
    /**/
    /*c0*/ NONE,
    /*c1*/ NONE,
    /*c2*/ NONE,
    /*c3*/ NONE,
    /*c4*/ NONE,
    /*c5*/ NONE,
    /*c6*/ NONE,
    /*c7*/ NONE,
    /*c8*/ NONE,
    /*c9*/ NONE,
    /*ca*/ NONE,
    /*cb*/ NONE,
    /*cc*/ NONE,
    /*cd*/ NONE,
    /*ce*/ NONE,
    /*cf*/ NONE,
    /**/
    /*d0*/ NONE,
    /*d1*/ NONE,
    /*d2*/ STP,
    /*d3*/ NONE,
    /*d4*/ NONE,
    /*d5*/ NONE,
    /*d6*/ NONE,
    /*d7*/ NONE,
    /*d8*/ NONE,
    /*d9*/ NONE,
    /*da*/ NONE,
    /*db*/ NONE,
    /*dc*/ NONE,
    /*dd*/ NONE,
    /*de*/ NONE,
    /*df*/ NONE,
    /**/
    /*e0*/ NONE,
    /*e1*/ NONE,
    /*e2*/ NONE,
    /*e3*/ NONE,
    /*e4*/ NONE,
    /*e5*/ NONE,
    /*e6*/ NONE,
    /*e7*/ NONE,
    /*e8*/ NONE,
    /*e9*/ NONE,
    /*ea*/ NOP,
    /*eb*/ NONE,
    /*ec*/ NONE,
    /*ed*/ NONE,
    /*ee*/ NONE,
    /*ef*/ NONE,
    /**/
    /*f0*/ NONE,
    /*f1*/ NONE,
    /*f2*/ STP,
    /*f3*/ NONE,
    /*f4*/ NONE,
    /*f5*/ NONE,
    /*f6*/ NONE,
    /*f7*/ NONE,
    /*f8*/ NONE,
    /*f9*/ NONE,
    /*fa*/ NONE,
    /*fb*/ NONE,
    /*fc*/ NONE,
    /*fd*/ NONE,
    /*fe*/ NONE,
    /*ff*/ NONE,
];
