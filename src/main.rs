mod audio;
mod chip_oxide;
mod cli;
mod constants;
mod keyboard;
mod machine;

use clap::Parser;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cli::Args;
use machine::Machine;

fn main() {
    let args = Arc::new(Args::parse());

    println!("Running with {}", args.file);

    let keyboard = Arc::new(Mutex::new(keyboard::Keyboard::new()));

    let machine = Arc::new(Mutex::new(Machine::new(
        Arc::clone(&args),
        Arc::clone(&keyboard),
    )));
    let machine_app_copy = Arc::clone(&machine);
    let machine_timer_copy = Arc::clone(&machine);

    machine.lock().unwrap().load_program(&args.file);

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

    let step_mode = args.step_mode;
    let machine_sleep_duration = 1_000_000_000 / args.cycles_per_second;
    // spawn machine thread
    thread::spawn(move || {
        loop {
            if !step_mode {
                machine.lock().unwrap().cycle();
                thread::sleep(Duration::from_nanos(machine_sleep_duration as u64));
            }
        }
    });

    chip_oxide::init(Arc::clone(&args), machine_app_copy, Arc::clone(&keyboard));
}
