use std::collections::HashSet;
use std::sync::{Arc, Condvar};

use eframe::egui::Key;

pub struct Keyboard {
    pub key_list: [bool; 16],
    /// An array of references to any condvars waiting for response
    pub wait_list: [Option<Arc<Condvar>>; 16],
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            key_list: [false; 16],
            wait_list: [const { None }; 16],
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

    pub fn set_wait(&mut self, key_num: usize) -> Arc<Condvar> {
        let cvar = Arc::new(Condvar::new());
        self.wait_list[key_num] = Some(Arc::clone(&cvar));
        cvar
    }

    pub fn release_wait(&mut self, key_num: usize) {
        match &self.wait_list[key_num] {
            Some(cvar) => cvar.notify_one(),
            None => panic!("Attempted release of {key_num} key failed"),
        };
    }
}
