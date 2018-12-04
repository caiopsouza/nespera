extern crate nespera;

use nespera::cpu::Cpu;
use nespera::loader::ines::INes;

fn main() {
    let rom = Vec::<u8>::from(&include_bytes!("../tests/resources/nestest.nes")[..]);

    let bus = &mut INes::new(rom).unwrap().into_bus();
    let mut cpu = Cpu::new(bus);
    cpu.reset();

    for frame in 0.. {
        for scanline in 0..=261 {
            for dot in 0..=340 {
                // Every third PPU cycle, run one cycle of the CPU
                if (scanline * 261 + dot) % 3 == 0 { cpu.step() }

                // Vblank
                if scanline == 241 && dot == 1 { cpu.bus.start_vblank() }
                if scanline == 261 && dot == 1 { cpu.bus.ppu.vblank_clear() }
            }
        }

        // Render the frame
        if frame == 100 {
            println!("frame: {}", frame);
            for y in 0..30 {
                for x in 0..32 {
                    let dot = cpu.bus.ppu.ram[0x2000 + y * 32 + x];
                    print!("{:02x} ", dot);
                }
                println!();
            }
            println!();
            println!("{}", cpu);
        }
    }
}
