mod audio;
mod chip_oxide;
mod cli;
mod constants;
mod instruction;
mod keyboard;
mod machine;
mod program_registry;

use clap::Parser;
use std::cell::RefCell;
use std::rc::Rc;

use cli::Args;
use machine::Machine;

fn main() {
    let reg = program_registry::ProgramRegistry::new();
    let rom = reg.get_rom_from_hash("5b733a60e7208f6aa0d15c99390ce4f670b2b886");
    println!("{:?}", rom.unwrap().platforms);

    // let args = Rc::new(RefCell::new(Args::parse()));
    // println!("Running with {}", args.borrow().file);

    //
    // let keyboard = Rc::new(RefCell::new(keyboard::Keyboard::new()));
    // let mut machine = Machine::new(Rc::clone(&args), Rc::clone(&keyboard));
    //
    // machine.load_program(&args.borrow().file);
    //
    // chip_oxide::init(Rc::clone(&args), machine, Rc::clone(&keyboard));
}
