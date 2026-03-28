use chip8::Chip8;

fn main() {
    let mut chip8 = Chip8::new();
    chip8.memory[0x200] = 0x12;
    chip8.memory[0x201] = 0xE4;
    chip8.print_mem_slice(0x200, 0x21F);
    println!("");
    chip8.print_registers();
    println!("\n");
    chip8.fetch();
    chip8.decode();
    chip8.execute();
    chip8.print_registers();
}
