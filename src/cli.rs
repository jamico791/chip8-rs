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

    /// activate shift quirk (vY as input if true, vX if false)
    #[arg(long)]
    pub shift: bool,

    /// activate jump quirk (XNN + vX if true, NNN + v0 if false)
    #[arg(long)]
    pub jump: bool,

    /// Set vF to 1 if I overflows to 0x1000 during Fx1E instruction if true
    #[arg(long)]
    pub fx1e_i_overflow: bool,

    /// Get key instruction happens on release if true, on press if false
    #[arg(long)]
    pub get_key_on_release: bool,

    /// The number of cycles per second
    #[arg(short, long, default_value_t = 700)]
    pub cycles_per_second: u32,
}
