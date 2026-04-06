mod audio;
mod chip_oxide;
mod cli;
mod constants;
mod instruction;
mod keyboard;
mod machine;

use clap::Parser;
use std::cell::RefCell;
use std::rc::Rc;

use cli::Args;
use machine::Machine;

fn main() {
    let args = Rc::new(RefCell::new(Args::parse()));
    println!("Running with {}", args.borrow().file);

    let keyboard = Rc::new(RefCell::new(keyboard::Keyboard::new()));
    let mut machine = Machine::new(Rc::clone(&args), Rc::clone(&keyboard));

    machine.load_program(&args.borrow().file);

    chip_oxide::init(Rc::clone(&args), machine, Rc::clone(&keyboard));
}
