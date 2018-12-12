// Describes a location on the console for reading and writing
// addr describes the address at the based of the array but have no upper bound.
#[derive(Debug, PartialEq)]
pub enum Location {
    // No location found for access.
    // The address is just for error reporting.
    Nowhere(u16),

    // APU. Not implemented yet.
    Apu(u16),

    // Addresses accessed by the CPU.
    CpuRam(u16),
    PpuCtrl,
    PpuMask,
    PpuStatus,
    OamAddr,
    OamData,
    PpuAddr,
    PpuScroll,
    PpuData,
    OamDma,

    // Addresses on the cartridge. Can be accessed by anyone.
    PrgRam(u16),
    PrgRom(u16),
    ChrRom(u16),
}
