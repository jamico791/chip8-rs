use std::collections::HashSet;

use eframe::egui::Key;

pub struct Keyboard {
    pub key_list: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            key_list: [false; 16],
        }
    }

    pub fn set_keys(&mut self, key_set: &HashSet<Key>) {
        self.key_list[0x0] = key_set.get(&Key::X).is_some();
        self.key_list[0x1] = key_set.get(&Key::Num1).is_some();
        self.key_list[0x2] = key_set.get(&Key::Num2).is_some();
        self.key_list[0x3] = key_set.get(&Key::Num3).is_some();
        self.key_list[0x4] = key_set.get(&Key::Q).is_some();
        self.key_list[0x5] = key_set.get(&Key::W).is_some();
        self.key_list[0x6] = key_set.get(&Key::E).is_some();
        self.key_list[0x7] = key_set.get(&Key::A).is_some();
        self.key_list[0x8] = key_set.get(&Key::S).is_some();
        self.key_list[0x9] = key_set.get(&Key::D).is_some();
        self.key_list[0xA] = key_set.get(&Key::Z).is_some();
        self.key_list[0xB] = key_set.get(&Key::C).is_some();
        self.key_list[0xC] = key_set.get(&Key::Num4).is_some();
        self.key_list[0xD] = key_set.get(&Key::R).is_some();
        self.key_list[0xE] = key_set.get(&Key::F).is_some();
        self.key_list[0xF] = key_set.get(&Key::V).is_some();
    }

    pub fn get_key(&self, key_num: usize) -> bool {
        self.key_list[key_num]
    }

    pub fn get_pressed(&self) -> Option<usize> {
        let mut i = 0;
        while i < 16 && !self.get_key(i) {
            i += 1;
        }
        if i < 16 { Some(i) } else { None }
    }
}
