mod audio;
mod chip_oxide;
mod cli;
mod constants;
mod keyboard;
mod machine;

use clap::Parser;
use eframe::egui::mutex::{Mutex, RwLock};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use cli::Args;
use machine::Machine;

fn main() {
    let args = Arc::new(RwLock::new(Args::parse()));
    let args_machine_copy = Arc::clone(&args);

    println!("Running with {}", args.read().file);

    let keyboard = Arc::new(Mutex::new(keyboard::Keyboard::new()));

    let machine = Arc::new(Mutex::new(Machine::new(
        Arc::clone(&args),
        Arc::clone(&keyboard),
    )));
    let machine_app_copy = Arc::clone(&machine);
    let machine_timer_copy = Arc::clone(&machine);

    machine.lock().load_program(&args.read().file);

    // spawn timer thread
    thread::spawn(move || {
        loop {
            {
                let mut c = machine_timer_copy.lock();
                if c.dt > 0 {
                    c.dt -= 1;
                }
                if c.st > 0 {
                    c.st -= 1;
                }
            }
            thread::sleep(Duration::from_nanos(16_666_667));
        }
    });

    let machine_sleep_duration = 1_000_000_000 / args.read().cycles_per_second;
    // spawn machine thread
    thread::spawn(move || {
        loop {
            if !args_machine_copy.read().step_mode {
                machine.lock().cycle();
                thread::sleep(Duration::from_nanos(machine_sleep_duration as u64));
            }
        }
    });

    chip_oxide::init(Arc::clone(&args), machine_app_copy, Arc::clone(&keyboard));
}
