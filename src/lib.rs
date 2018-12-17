#![feature(box_syntax)]
#![feature(core_intrinsics)]
#![feature(duration_float)]
#![feature(nll)]
#![feature(range_contains)]

extern crate env_logger;
#[macro_use]
extern crate log;

pub mod bus;
pub mod cartridge;
pub mod console;
pub mod cpu;
pub mod ppu;
pub mod ui;
pub mod utils;
