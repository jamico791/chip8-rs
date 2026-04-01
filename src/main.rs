mod chip_oxide;
mod cli;
mod constants;
mod machine;

use clap::Parser;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cli::Args;
use machine::Machine;

fn main() {
    let args = Args::parse();

    println!("Running with {}", args.file);

    let machine = Arc::new(Mutex::new(Machine::default()));
    let machine_app_copy = Arc::clone(&machine);
    let machine_timer_copy = Arc::clone(&machine);

    machine.lock().unwrap().load_program(&args.file);
    machine.lock().unwrap().dt = 255;

    // spawn timer thread
    thread::spawn(move || {
        loop {
            {
                let mut c = machine_timer_copy.lock().unwrap();
                if c.dt > 0 {
                    c.dt -= 1;
                }
                if c.st > 0 {
                    c.st -= 1;
                }
            }
            thread::sleep(Duration::from_nanos(16_670_000));
        }
    });

    // spawn machine thread
    thread::spawn(move || {
        loop {
            if !args.step_mode {
                machine.lock().unwrap().cycle();
                thread::sleep(Duration::from_nanos(1));
            }
        }
    });

    chip_oxide::init(args, machine_app_copy);
}
