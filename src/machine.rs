use eframe::egui::mutex::{Mutex, RwLock};
use std::sync::Arc;

pub use crate::constants::{FONT_START, MEMORY_LENGTH, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::keyboard::Keyboard;
use crate::{audio::Audio, cli::Args};

pub struct Machine {
    pub memory: [u8; 0x1000], // RAM
    pub screen_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub v: [u8; 0x10], // General purpose registers
    pub pc: u16,       // Program counter
    pub i: u16,        // Address register
    pub dt: u8,        // Delay timer register
    pub st: u8,        // Sound timer register
    pub instruction: Instruction,
    pub opcode: u16,
    pub stack: Vec<u16>,
    args: Arc<RwLock<Args>>,
    keyboard: Arc<Mutex<Keyboard>>,
    audio: Audio,
    waiting_for_key_release: Option<usize>,
}

impl Machine {
    pub fn new(args: Arc<RwLock<Args>>, keyboard: Arc<Mutex<Keyboard>>) -> Self {
        let mut machine = Self {
            memory: [0; 0x1000],
            screen_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v: [0; 0x10],
            pc: 0x200,
            i: 0,
            dt: 0,
            st: 0,
            instruction: Instruction::None,
            opcode: 0,
            stack: Vec::new(),
            args,
            keyboard,
            audio: Audio::new(220.0),
            waiting_for_key_release: None,
        };
        machine.inject_font();

        machine
    }

    pub fn fetch(&mut self) {
        let usize_pc = self.pc as usize;
        let left_byte = *self.memory.get(usize_pc).unwrap_or(&0) as u16;
        let right_byte = *self.memory.get(usize_pc + 1).unwrap_or(&0) as u16;

        self.opcode = (left_byte << 8) | right_byte;
        self.pc += 2;
    }

    fn decode(&mut self) {
        self.instruction = Instruction::new(self.opcode, self.args.read().jump);
    }

    pub fn execute(&mut self) {
        match self.instruction {
            Instruction::I00E0 => {
                self.screen_buffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            Instruction::I00EE => {
                self.pc = self
                    .stack
                    .pop()
                    .unwrap_or_else(|| panic!("Error when popping subroutine off stack"));
            }
            Instruction::I1NNN(nnn) => {
                self.pc = nnn;
            }
            Instruction::I2NNN(nnn) => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            Instruction::I3XNN(x, nn) => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }
            Instruction::I4XNN(x, nn) => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }
            Instruction::I5XY0(x, y) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            Instruction::I6XNN(x, nn) => {
                self.v[x] = nn;
            }
            Instruction::I7XNN(x, nn) => {
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            Instruction::I8XY0(x, y) => {
                self.v[x] = self.v[y];
            }
            Instruction::I8XY1(x, y) => {
                self.v[x] |= self.v[y];
            }
            Instruction::I8XY2(x, y) => {
                self.v[x] &= self.v[y];
            }
            Instruction::I8XY3(x, y) => {
                self.v[x] ^= self.v[y];
            }
            Instruction::I8XY4(x, y) => {
                let (result, did_overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = result;
                self.v[0xF] = if did_overflow { 1 } else { 0 };
            }
            Instruction::I8XY5(x, y) => {
                self.v[0xF] = if self.v[x] >= self.v[y] { 1 } else { 0 };
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
            }
            Instruction::I8XY6(x, y) => {
                if !self.args.read().shift {
                    self.v[x] = self.v[y];
                }
                self.v[0xF] = self.v[x] & 1;
                self.v[x] >>= 1;
            }
            Instruction::I8XY7(x, y) => {
                self.v[0xF] = if self.v[y] >= self.v[x] { 1 } else { 0 };
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
            }
            Instruction::I8XYE(x, y) => {
                if !self.args.read().shift {
                    self.v[x] = self.v[y];
                }
                self.v[0xF] = (self.v[x] & 0x80) >> 7;
                self.v[x] <<= 1;
            }
            Instruction::I9XY0(x, y) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            Instruction::IANNN(nnn) => {
                self.i = nnn;
            }
            Instruction::IBXNN(x, _, nnn) => {
                self.pc = nnn + self.v[x] as u16;
            }
            Instruction::IBNNN(nnn) => {
                self.pc = nnn + self.v[0] as u16;
            }
            Instruction::ICXNN(x, nn) => {
                let r: u8 = rand::random();
                self.v[x] = r & nn;
            }
            Instruction::IDXYN(x, y, n) => {
                let x_coord = (self.v[x] as usize) % SCREEN_WIDTH;
                let y_coord = (self.v[y] as usize) % SCREEN_HEIGHT;
                let sprite_vec = self.read_vector(self.i, n as u16);
                let mut had_collision = false;

                for (i, byte) in sprite_vec.iter().enumerate() {
                    let reverse_byte = byte.reverse_bits();
                    for j in 0..8 {
                        let adjusted_x = x_coord + j;
                        let adjusted_y = y_coord + i;
                        if adjusted_x < SCREEN_WIDTH && adjusted_y < SCREEN_HEIGHT {
                            let bit = (reverse_byte >> j) & 1;
                            if bit == 1 && !self.flip_pixel(adjusted_x, adjusted_y) {
                                had_collision = true;
                            }
                        }
                    }
                }
                self.v[0xF] = if had_collision { 1 } else { 0 };
            }
            Instruction::IEX9E(x) => {
                let key_is_pressed = self.keyboard.lock().get_key(self.v[x] as usize);
                if key_is_pressed {
                    self.pc += 2;
                }
            }
            Instruction::IEXA1(x) => {
                let key_is_not_pressed = !self.keyboard.lock().get_key(self.v[x] as usize);
                if key_is_not_pressed {
                    self.pc += 2;
                }
            }
            Instruction::IFX07(x) => {
                self.v[x] = self.dt;
            }
            Instruction::IFX15(x) => {
                self.dt = self.v[x];
            }
            Instruction::IFX18(x) => {
                self.st = self.v[x];
            }
            Instruction::IFX1E(x) => {
                let sum = self.i + self.v[x] as u16;
                if self.args.read().fx1e_i_overflow && sum > 0xFFF {
                    self.v[0xF] = 1
                }
                self.i = sum & 0xFFF;
            }
            Instruction::IFX0A(x) => {
                let kb = self.keyboard.lock();
                if self.args.read().get_key_on_release {
                    match self.waiting_for_key_release {
                        Some(key_num) => {
                            if kb.get_key(key_num) {
                                self.pc -= 2;
                            } else {
                                self.v[x] = key_num as u8;
                                self.waiting_for_key_release = None;
                            }
                        }
                        None => {
                            self.waiting_for_key_release = kb.get_pressed();
                            self.pc -= 2;
                        }
                    }
                } else {
                    if let Some(key_num) = kb.get_pressed() {
                        self.v[x] = key_num as u8;
                    } else {
                        self.pc -= 2;
                    }
                }
            }
            Instruction::IFX29(x) => {
                self.i = (FONT_START + (self.v[x] as usize * 5)) as u16;
            }
            Instruction::IFX33(x) => {
                if self.i >= 0xFFE {
                    panic!("Out of bounds memory access attempt from instruction FX33")
                }
                let ones = self.v[x] % 10;
                let tens = (self.v[x] / 10) % 10;
                let hundreds = self.v[x] / 100;

                self.memory[self.i as usize] = hundreds;
                self.memory[self.i as usize + 1] = tens;
                self.memory[self.i as usize + 2] = ones;
            }
            Instruction::IFX55(x) => {
                if self.i as usize > MEMORY_LENGTH - x {
                    panic!("Out of bounds memory access attempt from instruction FX55")
                }
                for i in 0..=x {
                    self.memory[self.i as usize + i] = self.v[i];
                }
                self.increment_i_for_quirks(x as u16);
            }
            Instruction::IFX65(x) => {
                if self.i as usize > MEMORY_LENGTH - x {
                    panic!("Out of bounds memory access attempt from instruction FX65")
                }
                for i in 0..=x {
                    self.v[i] = self.memory[self.i as usize + i];
                }
                self.increment_i_for_quirks(x as u16);
            }
            Instruction::None => panic!("Invalid instruction"),
        };
    }

    fn increment_i_for_quirks(&mut self, x: u16) {
        if !self.args.read().memory_leave_i_unchanged {
            self.i = if self.args.read().memory_increment_by_x {
                self.i + x
            } else {
                self.i + x + 1
            }
        }
    }

    fn flip_pixel(&mut self, x: usize, y: usize) -> bool {
        let i = (y * SCREEN_WIDTH) + x;
        self.screen_buffer[i] = !self.screen_buffer[i];
        self.screen_buffer[i]
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

        self.write_vector(font_vec, FONT_START);
    }

    pub fn load_program(&mut self, file: &String) {
        use std::fs::File;
        use std::io::{BufReader, Read};
        let f = File::open(file).unwrap_or_else(|e| panic!("File could not be opened: {e}"));
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();

        reader
            .read_to_end(&mut buffer)
            .unwrap_or_else(|e| panic!("Failed to read file: {e}"));

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
        self.decode();
        self.execute();
        self.set_beep();
    }

    fn set_beep(&mut self) {
        if self.st > 0 {
            self.audio.on();
        } else {
            self.audio.off();
        }
    }
}

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
    IBXNN(usize, u8, u16),
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
                    Instruction::IBXNN(x, nn, nnn)
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
            Instruction::I00E0 => write!(f, "CLS"),
            Instruction::I00EE => write!(f, "RET"),
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
            Instruction::IBXNN(x, _, nnn) => write!(f, "JP V{x:X} + {nnn:03X}"),
            Instruction::IBNNN(nnn) => write!(f, "JP V0 + {nnn:03X}"),
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
