use chip8::{Chip8, Instruction};
use clap::Parser;

/// A chip
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// the program input file
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();

    println!("Running with {}", args.file);

    let mut chip8 = Chip8::default();

    chip8.load_program(args.file);
    chip8.print_mem_slice(0x200, 0x2FF);
    println!();
    chip8.print_registers();

    chip8::display::init(chip8);
    // loop {
    //     chip8.fetch();
    //     chip8.decode_execute();
    //     chip8.print_registers();
    //     chip8.print_screen();
    //     if let Instruction::None = chip8.instruction {
    //         break;
    //     }
    // }
    
}
