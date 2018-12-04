use nespera::cpu::Cpu;
use nespera::loader::ines::INes;

mod nestest {
    use super::*;

    #[test]
    fn cpu() {
        let log = include_str!("../tests/resources/nestest.log");
        let rom = Vec::<u8>::from(&include_bytes!("../tests/resources/nestest.nes")[..]);
        let bus = &mut INes::new(rom).unwrap().into_bus();
        let mut cpu = Cpu::new(bus);
        cpu.set_clock(0);

        // Starting point where the ROM won't access the PPU.
        cpu.reg.set_pc(0xc000);

        for (_, text) in log.split("\r\n").enumerate() {
            let ppu_cycle = (3 * cpu.get_clock()) % 341;

            let p: u8 = cpu.reg.get_p().into();
            let res = format!("{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{: >3}",
                              cpu.reg.get_pc(), cpu.reg.get_a(), cpu.reg.get_x(),
                              cpu.reg.get_y(), p, cpu.reg.get_s(), ppu_cycle);

            assert_eq!(res, text, "\n\n{}", cpu);

            cpu.step_instruction()
        }

        // Return from subroutine
        assert_eq!(0x60, cpu.reg.get_current_instr(), "\n\n{}", cpu);
    }
}
