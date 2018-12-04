extern crate nespera;

use nespera::cpu::Cpu;
use nespera::loader::ines::INes;

fn main() {
    let rom = Vec::<u8>::from(&include_bytes!("../tests/resources/sprites.nes")[..]);
    let bus = &mut INes::new(rom).unwrap().into_bus();
    let mut cpu = Cpu::new(bus);
    cpu.reset();

    // Run the CPU and simulate a VBlank after some cycles
    loop {
        // Break here because the ROM is in a infinite loop.
        if cpu.reg.get_current_instr() == 0x4C { break; }

        for _ in 0..100000 {
            cpu.step_instruction();
        }

        cpu.bus.start_vblank();
    }

    eprintln!("{}", cpu);
}
