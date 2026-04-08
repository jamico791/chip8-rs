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

    /// The number of instructions per frame
    #[arg(short, long = "ipf", default_value_t = 20)]
    pub instructions_per_frame: u32,

    /// FX55 & FX65 increment I with X if true, and with X + 1 if false
    #[arg(long)]
    pub memory_increment_by_x: bool,

    /// FX55 & FX65 leave the I register unchanged if true, and increment it if false
    #[arg(long)]
    pub memory_leave_i_unchanged: bool,

    /// Draw instruction will wait until the next frame boundary move on if true
    #[arg(long)]
    pub vblank: bool,

    /// 8XY1/2/3 set vF to 0 if true, and leave it unchanged if false
    #[arg(long)]
    pub logic: bool,
}
