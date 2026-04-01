use clap::Parser;

/// A Chip-8 Interpreter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// the program input file
    #[arg(short, long)]
    pub file: String,

    /// activate debug mode
    #[arg(short, long)]
    pub debug: bool,

    /// activate step mode
    #[arg(short, long)]
    pub step_mode: bool,
}
