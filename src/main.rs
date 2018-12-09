use std::cell::RefCell;
use std::rc::Rc;

use nespera::bus::Bus;
use nespera::cpu::Cpu;
use nespera::mapper::Cartridge;

fn main() {
    env_logger::init();

    let file = include_bytes!("../tests/resources/cpu/nestest.nes")[..].to_owned();
    let cartridge = Cartridge::new(file).unwrap();
    let bus = Rc::new(RefCell::new(Bus::with_cartridge(cartridge)));
    let mut _cpu = Cpu::new(bus);
    /*cpu.reset();

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
    }*/
}
