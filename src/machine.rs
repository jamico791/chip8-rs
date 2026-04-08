use std::cell::RefCell;
use std::rc::Rc;

use crate::constants::{FONT_START, MEMORY_LENGTH, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::instruction::Instruction;
use crate::keyboard::Keyboard;
use crate::{audio::Audio, cli::Args};

pub struct Machine {
    pub memory: [u8; 0x1000], // RAM
    pub front_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub back_buffer: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub v: [u8; 0x10], // General purpose registers
    pub pc: u16,       // Program counter
    pub i: u16,        // Address register
    pub dt: u8,        // Delay timer register
    pub st: u8,        // Sound timer register
    pub instruction: Instruction,
    pub opcode: u16,
    pub stack: Vec<u16>,
    args: Rc<RefCell<Args>>,
    keyboard: Rc<RefCell<Keyboard>>,
    audio: Audio,
    waiting_for_key_release: Option<usize>,
    pub is_next_frame: bool,
}

impl Machine {
    pub fn new(args: Rc<RefCell<Args>>, keyboard: Rc<RefCell<Keyboard>>) -> Self {
        let mut machine = Self {
            memory: [0; 0x1000],
            front_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            back_buffer: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
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
            is_next_frame: false,
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
        self.instruction = Instruction::new(self.opcode, self.args.borrow().jump);
    }

    pub fn execute(&mut self) -> Option<i32> {
        match self.instruction {
            Instruction::I00E0 => {
                self.back_buffer = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
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
                self.logic_quirk();
            }
            Instruction::I8XY2(x, y) => {
                self.v[x] &= self.v[y];
                self.logic_quirk();
            }
            Instruction::I8XY3(x, y) => {
                self.v[x] ^= self.v[y];
                self.logic_quirk();
            }
            Instruction::I8XY4(x, y) => {
                let (result, did_overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = result;
                self.v[0xF] = if did_overflow { 1 } else { 0 };
            }
            Instruction::I8XY5(x, y) => {
                let v_x = self.v[x];
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                self.v[0xF] = if v_x >= self.v[y] { 1 } else { 0 };
            }
            Instruction::I8XY6(x, y) => {
                if !self.args.borrow().shift {
                    self.v[x] = self.v[y];
                }
                let v_x = self.v[x];
                self.v[x] >>= 1;
                self.v[0xF] = v_x & 1;
            }
            Instruction::I8XY7(x, y) => {
                let v_x = self.v[x];
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                self.v[0xF] = if self.v[y] >= v_x { 1 } else { 0 };
            }
            Instruction::I8XYE(x, y) => {
                if !self.args.borrow().shift {
                    self.v[x] = self.v[y];
                }
                let v_x = self.v[x];
                self.v[x] <<= 1;
                self.v[0xF] = (v_x & 0x80) >> 7;
            }
            Instruction::I9XY0(x, y) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            Instruction::IANNN(nnn) => {
                self.i = nnn;
            }
            Instruction::IBXNN(x, nnn) => {
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
                if self.args.borrow().vblank {
                    if !self.is_next_frame {
                        self.is_next_frame = true;
                        self.pc -= 2;
                        return Some(1);
                    } else {
                        self.is_next_frame = false;
                    }
                }

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
                let key_is_pressed = self.keyboard.borrow().get_key(self.v[x] as usize);
                if key_is_pressed {
                    self.pc += 2;
                }
            }
            Instruction::IEXA1(x) => {
                let key_is_not_pressed = !self.keyboard.borrow().get_key(self.v[x] as usize);
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
                if self.args.borrow().fx1e_i_overflow && sum > 0xFFF {
                    self.v[0xF] = 1
                }
                self.i = sum & 0xFFF;
            }
            Instruction::IFX0A(x) => {
                let kb = self.keyboard.borrow_mut();
                if self.args.borrow().get_key_on_release {
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

        None
    }

    fn increment_i_for_quirks(&mut self, x: u16) {
        if !self.args.borrow().memory_leave_i_unchanged {
            self.i = if self.args.borrow().memory_increment_by_x {
                self.i + x
            } else {
                self.i + x + 1
            }
        }
    }

    fn logic_quirk(&mut self) {
        if self.args.borrow().logic {
            self.v[0xF] = 0;
        }
    }

    fn flip_pixel(&mut self, x: usize, y: usize) -> bool {
        let i = (y * SCREEN_WIDTH) + x;
        self.back_buffer[i] = !self.back_buffer[i];
        self.back_buffer[i]
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

    pub fn get_display_buffer(&self) -> &[bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        &self.front_buffer
    }

    /// Copy back buffer to front buffer, not actually swapping anything
    pub fn swap_buffers(&mut self) {
        self.front_buffer.copy_from_slice(&self.back_buffer);
    }

    pub fn get_memory(&self) -> &[u8; MEMORY_LENGTH] {
        &self.memory
    }

    pub fn decrement_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    pub fn cycle(&mut self) -> i32 {
        self.fetch();
        self.decode();

        // if execute exits early, return the code
        let return_code = self.execute();
        if let Some(code) = return_code {
            return code;
        }

        self.set_beep();

        0
    }

    fn set_beep(&mut self) {
        if self.st > 0 {
            self.audio.on();
        } else {
            self.audio.off();
        }
    }
}
