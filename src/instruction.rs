#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum Instruction {
    I00E0,
    I00EE,
    I1NNN(u16),
    I2NNN(u16),
    I3XNN(usize, u8),
    I4XNN(usize, u8),
    I5XY0(usize, usize),
    I6XNN(usize, u8),
    I7XNN(usize, u8),
    I8XY0(usize, usize),
    I8XY1(usize, usize),
    I8XY2(usize, usize),
    I8XY3(usize, usize),
    I8XY4(usize, usize),
    I8XY5(usize, usize),
    I8XY6(usize, usize),
    I8XY7(usize, usize),
    I8XYE(usize, usize),
    I9XY0(usize, usize),
    IANNN(u16),
    IBXNN(usize, u16),
    IBNNN(u16),
    ICXNN(usize, u8),
    IDXYN(usize, usize, u8),
    IEX9E(usize),
    IEXA1(usize),
    IFX07(usize),
    IFX0A(usize),
    IFX15(usize),
    IFX18(usize),
    IFX1E(usize),
    IFX29(usize),
    IFX33(usize),
    IFX55(usize),
    IFX65(usize),
    None,
}

impl Instruction {
    pub fn new(opcode: u16, jump: bool) -> Instruction {
        let x: usize = ((opcode & 0xF00) >> 8) as usize;
        let y: usize = ((opcode & 0xF0) >> 4) as usize;
        let n: u8 = (opcode & 0xF) as u8;
        let nn: u8 = (opcode & 0xFF) as u8;
        let nnn: u16 = opcode & 0xFFF;

        let tup = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            opcode & 0x000F,
        );

        match tup {
            (0x0, 0x0, 0xE, 0x0) => Instruction::I00E0,
            (0x0, 0x0, 0xE, 0xE) => Instruction::I00EE,
            (0x1, _, _, _) => Instruction::I1NNN(nnn),
            (0x2, _, _, _) => Instruction::I2NNN(nnn),
            (0x3, _, _, _) => Instruction::I3XNN(x, nn),
            (0x4, _, _, _) => Instruction::I4XNN(x, nn),
            (0x5, _, _, 0x0) => Instruction::I5XY0(x, y),
            (0x6, _, _, _) => Instruction::I6XNN(x, nn),
            (0x7, _, _, _) => Instruction::I7XNN(x, nn),
            (0x8, _, _, 0x0) => Instruction::I8XY0(x, y),
            (0x8, _, _, 0x1) => Instruction::I8XY1(x, y),
            (0x8, _, _, 0x2) => Instruction::I8XY2(x, y),
            (0x8, _, _, 0x3) => Instruction::I8XY3(x, y),
            (0x8, _, _, 0x4) => Instruction::I8XY4(x, y),
            (0x8, _, _, 0x5) => Instruction::I8XY5(x, y),
            (0x8, _, _, 0x6) => Instruction::I8XY6(x, y),
            (0x8, _, _, 0x7) => Instruction::I8XY7(x, y),
            (0x8, _, _, 0xE) => Instruction::I8XYE(x, y),
            (0x9, _, _, 0x0) => Instruction::I9XY0(x, y),
            (0xA, _, _, _) => Instruction::IANNN(nnn),
            (0xB, _, _, _) => {
                if jump {
                    Instruction::IBXNN(x, nnn)
                } else {
                    Instruction::IBNNN(nnn)
                }
            }
            (0xC, _, _, _) => Instruction::ICXNN(x, nn),
            (0xD, _, _, _) => Instruction::IDXYN(x, y, n),
            (0xE, _, 0x9, 0xE) => Instruction::IEX9E(x),
            (0xE, _, 0xA, 0x1) => Instruction::IEXA1(x),
            (0xF, _, 0x0, 0x7) => Instruction::IFX07(x),
            (0xF, _, 0x1, 0x5) => Instruction::IFX15(x),
            (0xF, _, 0x1, 0x8) => Instruction::IFX18(x),
            (0xF, _, 0x1, 0xE) => Instruction::IFX1E(x),
            (0xF, _, 0x0, 0xA) => Instruction::IFX0A(x),
            (0xF, _, 0x2, 0x9) => Instruction::IFX29(x),
            (0xF, _, 0x3, 0x3) => Instruction::IFX33(x),
            (0xF, _, 0x5, 0x5) => Instruction::IFX55(x),
            (0xF, _, 0x6, 0x5) => Instruction::IFX65(x),
            (_, _, _, _) => Instruction::None,
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::I00E0 => write!(f, "clear"),
            Instruction::I00EE => write!(f, "return"),
            Instruction::I1NNN(nnn) => write!(f, "jump {nnn:#05X}"),
            Instruction::I2NNN(nnn) => write!(f, ":call {nnn:#05X}"),
            Instruction::I3XNN(x, nn) => write!(f, "if v{x:X} != {nn:#04X} then"),
            Instruction::I4XNN(x, nn) => write!(f, "if v{x:X} == {nn:#04X} then"),
            Instruction::I5XY0(x, y) => write!(f, "if v{x:X} != v{y:X} then"),
            Instruction::I6XNN(x, nn) => write!(f, "v{x:X} := {nn:#04X}"),
            Instruction::I7XNN(x, nn) => write!(f, "v{x:X} += {nn:#04X}"),
            Instruction::I8XY0(x, y) => write!(f, "v{x:X} := v{y:X}"),
            Instruction::I8XY1(x, y) => write!(f, "v{x:X} |= v{y:X}"),
            Instruction::I8XY2(x, y) => write!(f, "v{x:X} &= v{y:X}"),
            Instruction::I8XY3(x, y) => write!(f, "v{x:X} ^= v{y:X}"),
            Instruction::I8XY4(x, y) => write!(f, "v{x:X} += v{y:X}"),
            Instruction::I8XY5(x, y) => write!(f, "v{x:X} -= v{y:X}"),
            Instruction::I8XY6(x, y) => write!(f, "v{x:X} >>= v{y:X}"),
            Instruction::I8XY7(x, y) => write!(f, "v{x:X} =- v{y:X}"),
            Instruction::I8XYE(x, y) => write!(f, "v{x:X} <<= v{y:X}"),
            Instruction::I9XY0(x, y) => write!(f, "if v{x:X} == v{y:X} then"),
            Instruction::IANNN(nnn) => write!(f, "i := {nnn:#05X}"),
            Instruction::IBXNN(x, nnn) => write!(f, "jump{x:X} {nnn:#05X}"),
            Instruction::IBNNN(nnn) => write!(f, "jump0 {nnn:#05X}"),
            Instruction::ICXNN(x, nn) => write!(f, "v{x:X} := random {nn:#04X}"),
            Instruction::IDXYN(x, y, n) => write!(f, "sprite v{x:X} v{y:X} {n:#03X}"),
            Instruction::IEX9E(x) => write!(f, "if v{x:X} -key then"),
            Instruction::IEXA1(x) => write!(f, "if v{x:X} key then"),
            Instruction::IFX07(x) => write!(f, "v{x:X} := delay"),
            Instruction::IFX0A(x) => write!(f, "v{x:X} := key"),
            Instruction::IFX15(x) => write!(f, "delay := v{x:X}"),
            Instruction::IFX18(x) => write!(f, "buzzer := v{x:X}"),
            Instruction::IFX1E(x) => write!(f, "i += v{x:X}"),
            Instruction::IFX29(x) => write!(f, "i := hex v{x:X}"),
            Instruction::IFX33(x) => write!(f, "bcd v{x:X}"),
            Instruction::IFX55(x) => write!(f, "save v{x:X}"),
            Instruction::IFX65(x) => write!(f, "load v{x:X}"),
            Instruction::None => write!(f, ""),
        }
    }
}
