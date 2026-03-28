const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

pub struct Chip8 {
    pub memory: [u8; 0x1000],   // RAM
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub v: [u8; 0x10],          // General purpose registers
    pub pc: u16,                // Program counter
    pub i: u16,                 // Address register
    pub dt: u8,                 // Delay timer register
    pub st: u8,                 // Sound timer register
    pub instruction: Instruction,
    pub opcode: u16,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; 0x1000],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v: [0; 0x10],
            pc: 0x200,
            i: 0,
            dt: 0,
            st: 0,
            instruction: Instruction::None,
            opcode: 0,
        }
    }

    pub fn print_mem_slice(&self, start: usize, end: usize) {
        let sub_slice = self.memory
                            .get(start..=end)
                            .unwrap_or_else(|| panic!("print_mem_slice: Invalid start and/or end"));

        for (i, byte) in sub_slice.iter().enumerate() {
            if (i + 16) % 16 == 0 {
                print!("{:03X}  ", i + start);
            }
            print!("{:02X}", byte);
            if (i + 1) % 16 == 0 {
                print!("\n");
            } else if (i + 1) % 2 == 0 {
                print!(" ");
            }
        }
    }

    pub fn print_mem(&self) {
        self.print_mem_slice(0, self.memory.len() - 1);
    }

    fn print_v(&self) {
        for i in 0..self.v.len() {
            print!("V{i:X}: {:02}", self.v[i]);
            if (i + 1) % 2 == 0 && i != 0  && i != self.v.len() - 1 {
                print!("\n")
            } else {
                print!(" ");
            }
        }
    }

    fn print_special_registers(&self) {
        println!("I: {:03X} DT: {:02X} ST: {:02X}", self.i, self.dt, self.st);
    }

    fn print_instruction(&self) {
        println!("PC: {:03X} Opcode: {:04X} Instruction: {}", self.pc, self.opcode, self.instruction);
    }

    pub fn print_registers(&self) {
        self.print_instruction();
        self.print_special_registers();
        println!("");
        self.print_v();
    }

    pub fn fetch(&mut self) {
        let usize_pc = self.pc as usize;
        let left_byte = *self.memory.get(usize_pc).unwrap_or_else(|| &0) as u16;
        let right_byte = *self.memory.get(usize_pc + 1).unwrap_or_else(|| &0) as u16;

        self.opcode = (left_byte << 8) | right_byte;
        self.pc += 1;
    }

    pub fn decode(&mut self) {
        let tup = ((self.opcode & 0xF000) >> 12, (self.opcode & 0x0F00) >> 8, (self.opcode & 0x00F0) >> 4, self.opcode & 0x000F);
        self.instruction = match tup {
            (0x0, 0x0, 0xE, 0x0) => Instruction::I00E0,
            (0x0, 0x0, 0xE, 0xE) => Instruction::I00EE,
            (0x1, _, _, _) => Instruction::I1NNN(self.opcode & 0xFFF),
            (_, _, _, _) => Instruction::None,
        };
    }

    pub fn execute(&self) {
        
    }
}

#[derive(Debug)]
pub enum Instruction {
    I0NNN(u16),
    I00E0,
    I00EE,
    I1NNN(u16),
    I2NNN(u16),
    I3XNN(u8, u8),
    I4XNN(u8, u8),
    I5XY0(u8, u8),
    I6XNN(u8, u8),
    I7XNN(u8, u8),
    I8XY0(u8, u8),
    I8XY1(u8, u8),
    I8XY2(u8, u8),
    I8XY3(u8, u8),
    I8XY4(u8, u8),
    I8XY5(u8, u8),
    I8XY6(u8, u8),
    I8XY7(u8, u8),
    I8XYE(u8, u8),
    I9XY0(u8, u8),
    IANNN(u16),
    IBNNN(u16),
    ICXNN(u8, u8),
    IDXYN(u8, u8),
    IEX9E(u8),
    IEXA1(u8),
    IFX07(u8),
    IFX0A(u8),
    IFX15(u8),
    IFX18(u8),
    IFX1E(u8),
    IFX29(u8),
    IFX33(u8),
    IFX55(u8),
    IFX65(u8),
    None,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::I00E0        => write!(f, "CLS"),
            Instruction::I00EE        => write!(f, "RET"),
            Instruction::I0NNN(nnn)   => write!(f, "SYS {nnn:03X}"),
            Instruction::I1NNN(nnn)   => write!(f, "JP {nnn:03X}"),
            Instruction::I2NNN(nnn)   => write!(f, "CALL {nnn:03X}"),
            Instruction::I3XNN(x, nn) => write!(f, "SE V{x:X}, {nn:02X}"),
            Instruction::I4XNN(x, nn) => write!(f, "SNE V{x:X}, {nn:02X}"),
            Instruction::I5XY0(x, y)  => write!(f, "SE V{x:X}, V{y:X}"),
            Instruction::I6XNN(x, nn) => write!(f, "LD V{x:X}, {nn:02X}"),
            Instruction::I7XNN(x, nn) => write!(f, "ADD V{x:X}, {nn:02X}"),
            Instruction::I8XY0(x, y)  => write!(f, "LD V{x:X}, V{y:X}"),
            Instruction::I8XY1(x, y)  => write!(f, "OR V{x:X}, V{y:X}"),
            Instruction::I8XY2(x, y)  => write!(f, "AND V{x:X}, V{y:X}"),
            Instruction::I8XY3(x, y)  => write!(f, "XOR V{x:X}, V{y:X}"),
            Instruction::I8XY4(x, y)  => write!(f, "ADD V{x:X}, V{y:X}"),
            Instruction::I8XY5(x, y)  => write!(f, "SUB V{x:X}, V{y:X}"),
            Instruction::I8XY6(x, y)  => write!(f, "SHR V{x:X}, V{y:X}"),
            Instruction::I8XY7(x, y)  => write!(f, "SUBN V{x:X}, V{y:X}"),
            Instruction::I8XYE(x, y)  => write!(f, "SHL V{x:X}, V{y:X}"),
            Instruction::I9XY0(x, y)  => write!(f, "SNE V{x:X}, V{y:X}"),
            Instruction::IANNN(nnn)   => write!(f, "LD I, {nnn:03X}"),
            Instruction::IBNNN(nnn)   => write!(f, "JP V0, {nnn:03X}"),
            Instruction::ICXNN(x, nn) => write!(f, "RND V{x:X}, {nn:02X}"),
            Instruction::IDXYN(x, y)  => write!(f, "DRW V{x:X}, V{y:X}"),
            Instruction::IEX9E(x)     => write!(f, "SKP V{x:X}"),
            Instruction::IEXA1(x)     => write!(f, "SKNP V{x:X}"),
            Instruction::IFX07(x)     => write!(f, "LD V{x:X}, DT"),
            Instruction::IFX0A(x)     => write!(f, "LD V{x:X}, K"),
            Instruction::IFX15(x)     => write!(f, "LD DT, V{x:X}"),
            Instruction::IFX18(x)     => write!(f, "LD ST, V{x:X}"),
            Instruction::IFX1E(x)     => write!(f, "ADD I, V{x:X}"),
            Instruction::IFX29(x)     => write!(f, "LD F, V{x:X}"),
            Instruction::IFX33(x)     => write!(f, "LD B, V{x:X}"),
            Instruction::IFX55(x)     => write!(f, "LD [I], V{x:X}"),
            Instruction::IFX65(x)     => write!(f, "LD V{x:X}, [I]"),
            Instruction::None         => write!(f, "NONE"),
        }
    }
}

// trait UnsignedInt {}

// impl UnsignedInt for u8 {}
// impl UnsignedInt for u16 {}
// impl UnsignedInt for u32 {}
// impl UnsignedInt for u64 {}
// impl UnsignedInt for u128 {}
// impl UnsignedInt for usize {}
