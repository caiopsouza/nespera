#![feature(nll)]
#![feature(box_syntax)]
#![feature(range_contains)]
#![feature(duration_float)]

extern crate env_logger;
#[macro_use]
extern crate log;

pub mod bus;
pub mod cartridge;
pub mod console;
pub mod cpu;
pub mod ui;
pub mod utils;
