pub mod display;
mod constants;

use constants::{MEMORY_LENGTH, SCREEN_WIDTH, SCREEN_HEIGHT};

pub struct Chip8 {
    pub memory: [u8; 0x1000], // RAM
    pub screen_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub v: [u8; 0x10], // General purpose registers
    pub pc: u16,       // Program counter
    pub i: u16,        // Address register
    pub dt: u8,        // Delay timer register
    pub st: u8,        // Sound timer register
    pub instruction: Instruction,
    pub opcode: u16,
}

impl Chip8 {
    pub fn init(&mut self) {
        self.inject_font();
    }

    pub fn print_mem_slice(&self, start: usize, end: usize) {
        let sub_slice = self
            .memory
            .get(start..=end)
            .unwrap_or_else(|| panic!("print_mem_slice: Invalid start and/or end"));

        for (i, byte) in sub_slice.iter().enumerate() {
            if (i + 16) % 16 == 0 {
                print!("{:03X}  ", i + start);
            }
            print!("{:02X}", byte);
            if (i + 1) % 16 == 0 {
                println!();
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
            print!("V{i:X}: {:02X}", self.v[i]);
            if (i + 1) % 2 == 0 && i != 0 && i != self.v.len() - 1 {
                println!()
            } else {
                print!(" ");
            }
        }
        println!();
    }

    fn print_special_registers(&self) {
        println!("I: {:03X} DT: {:02X} ST: {:02X}", self.i, self.dt, self.st);
    }

    fn print_instruction(&self) {
        println!(
            "PC: {:03X} Opcode: {:04X} Instruction: {}",
            self.pc, self.opcode, self.instruction
        );
    }

    pub fn print_registers(&self) {
        self.print_instruction();
        self.print_special_registers();
        println!();

        self.print_v();
    }

    pub fn fetch(&mut self) {
        let usize_pc = self.pc as usize;
        let left_byte = *self.memory.get(usize_pc).unwrap_or_else(|| &0) as u16;
        let right_byte = *self.memory.get(usize_pc + 1).unwrap_or_else(|| &0) as u16;

        self.opcode = (left_byte << 8) | right_byte;
        self.pc += 2;
    }

    pub fn decode_execute(&mut self) {
        let x: u8 = ((self.opcode & 0xF00) >> 8) as u8;
        let y: u8 = ((self.opcode & 0xF0) >> 4) as u8;
        let n: u8 = (self.opcode & 0xF) as u8;
        let nn: u8 = (self.opcode & 0xFF) as u8;
        let nnn: u16 = self.opcode & 0xFFF;

        let tup = (
            (self.opcode & 0xF000) >> 12,
            (self.opcode & 0x0F00) >> 8,
            (self.opcode & 0x00F0) >> 4,
            self.opcode & 0x000F,
        );

        self.instruction = match tup {
            (0x0, 0x0, 0xE, 0x0) => {
                self.screen_buffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
                Instruction::I00E0
            }
            (0x0, 0x0, 0xE, 0xE) => Instruction::I00EE,
            (0x1, _, _, _) => {
                self.pc = nnn;
                Instruction::I1NNN(nnn)
            }
            (0x6, _, _, _) => {
                self.v[x as usize] = nn;
                Instruction::I6XNN(x, nn)
            }
            (0x7, _, _, _) => {
                self.v[x as usize] += nn;
                Instruction::I7XNN(x, nn)
            }
            (0xA, _, _, _) => {
                self.i = nnn;
                Instruction::IANNN(nnn)
            }
            (0xD, _, _, _) => {
                let x_coord = (self.v[x as usize] as usize) % SCREEN_WIDTH;
                let y_coord = (self.v[y as usize] as usize) % SCREEN_HEIGHT;
                let sprite_vec = self.read_vector(self.i, n as u16);
                let mut had_collision = false;
                println!("x: {x_coord} y: {y_coord} sprite_vec: {sprite_vec:?}");

                for (i, byte) in sprite_vec.iter().enumerate() {
                    let reverse_byte = byte.reverse_bits();
                    for j in 0..8 {
                        let adjusted_x = x_coord + j;
                        let adjusted_y = y_coord + i;
                        if adjusted_x < SCREEN_WIDTH && adjusted_y < SCREEN_HEIGHT {
                            let bit = (reverse_byte >> j) & 1;
                            if bit == 1 {
                                if !self.flip_pixel(adjusted_x, adjusted_y) {
                                    had_collision = true;
                                }
                            }
                        }
                    }
                }
                self.v[0xF] = if had_collision { 1 } else { 0 };
                Instruction::IDXYN(x, y, n)
            }
            (_, _, _, _) => Instruction::None,
        };
    }

    fn flip_pixel(&mut self, x: usize, y: usize) -> bool {
        let i = (y * SCREEN_WIDTH) + x;
        self.screen_buffer[i] = !self.screen_buffer[i];
        return self.screen_buffer[i];
    }

    pub fn read_vector(&self, start: u16, length: u16) -> Vec<u8> {
        let mut v = Vec::new();

        for i in start..start + length {
            v.push(self.memory[i as usize]);
        }

        v
    }

    pub fn write_vector(&mut self, v: Vec<u8>, start: usize) {
        for (i, num) in v.iter().enumerate() {
            self.memory[i + start] = *num;
        }
    }

    pub fn inject_font(&mut self) {
        let font_vec = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        self.write_vector(font_vec, 0x050);
    }

    pub fn print_screen(&self) {
        for (i, pixel) in self.screen_buffer.iter().enumerate() {
            if *pixel {
                print!("█");
            } else {
                print!(" ")
            }
            if (i + 1) % SCREEN_WIDTH == 0 {
                println!();
            }
        }
    }

    pub fn load_program(&mut self, file: String) {
        use std::io::{Read, BufReader};
        use std::fs::File;
        let f = File::open(file).unwrap_or_else(|e| panic!("File could not be opened: {e}"));
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        
        reader.read_to_end(&mut buffer).unwrap_or_else(|e| panic!("Failed to read file: {e}"));

        self.write_vector(buffer, 0x200);
    }

    pub fn get_screen_buffer(&self) -> &[bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.screen_buffer
    }

    pub fn get_memory(&self) -> &[u8; MEMORY_LENGTH] {
        &self.memory
    }

    pub fn cycle(&mut self) {
        self.fetch();
        self.decode_execute();
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut chip8 = Chip8 {
            memory: [0; 0x1000],
            screen_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v: [0; 0x10],
            pc: 0x200,
            i: 0,
            dt: 0,
            st: 0,
            instruction: Instruction::None,
            opcode: 0,
        };
        chip8.init();

        chip8
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
    IDXYN(u8, u8, u8),
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
            Instruction::I00E0 => write!(f, "CLS"),
            Instruction::I00EE => write!(f, "RET"),
            Instruction::I0NNN(nnn) => write!(f, "SYS {nnn:03X}"),
            Instruction::I1NNN(nnn) => write!(f, "JP {nnn:03X}"),
            Instruction::I2NNN(nnn) => write!(f, "CALL {nnn:03X}"),
            Instruction::I3XNN(x, nn) => write!(f, "SE V{x:X}, {nn:02X}"),
            Instruction::I4XNN(x, nn) => write!(f, "SNE V{x:X}, {nn:02X}"),
            Instruction::I5XY0(x, y) => write!(f, "SE V{x:X}, V{y:X}"),
            Instruction::I6XNN(x, nn) => write!(f, "LD V{x:X}, {nn:02X}"),
            Instruction::I7XNN(x, nn) => write!(f, "ADD V{x:X}, {nn:02X}"),
            Instruction::I8XY0(x, y) => write!(f, "LD V{x:X}, V{y:X}"),
            Instruction::I8XY1(x, y) => write!(f, "OR V{x:X}, V{y:X}"),
            Instruction::I8XY2(x, y) => write!(f, "AND V{x:X}, V{y:X}"),
            Instruction::I8XY3(x, y) => write!(f, "XOR V{x:X}, V{y:X}"),
            Instruction::I8XY4(x, y) => write!(f, "ADD V{x:X}, V{y:X}"),
            Instruction::I8XY5(x, y) => write!(f, "SUB V{x:X}, V{y:X}"),
            Instruction::I8XY6(x, y) => write!(f, "SHR V{x:X}, V{y:X}"),
            Instruction::I8XY7(x, y) => write!(f, "SUBN V{x:X}, V{y:X}"),
            Instruction::I8XYE(x, y) => write!(f, "SHL V{x:X}, V{y:X}"),
            Instruction::I9XY0(x, y) => write!(f, "SNE V{x:X}, V{y:X}"),
            Instruction::IANNN(nnn) => write!(f, "LD I, {nnn:03X}"),
            Instruction::IBNNN(nnn) => write!(f, "JP V0, {nnn:03X}"),
            Instruction::ICXNN(x, nn) => write!(f, "RND V{x:X}, {nn:02X}"),
            Instruction::IDXYN(x, y, n) => write!(f, "DRW V{x:X}, V{y:X}, {n}"),
            Instruction::IEX9E(x) => write!(f, "SKP V{x:X}"),
            Instruction::IEXA1(x) => write!(f, "SKNP V{x:X}"),
            Instruction::IFX07(x) => write!(f, "LD V{x:X}, DT"),
            Instruction::IFX0A(x) => write!(f, "LD V{x:X}, K"),
            Instruction::IFX15(x) => write!(f, "LD DT, V{x:X}"),
            Instruction::IFX18(x) => write!(f, "LD ST, V{x:X}"),
            Instruction::IFX1E(x) => write!(f, "ADD I, V{x:X}"),
            Instruction::IFX29(x) => write!(f, "LD F, V{x:X}"),
            Instruction::IFX33(x) => write!(f, "LD B, V{x:X}"),
            Instruction::IFX55(x) => write!(f, "LD [I], V{x:X}"),
            Instruction::IFX65(x) => write!(f, "LD V{x:X}, [I]"),
            Instruction::None => write!(f, "NONE"),
        }
    }
}
