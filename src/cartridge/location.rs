// Describes a location on the console for reading and writing
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Location {
    // No location found for access. Used for error reporting only.
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
