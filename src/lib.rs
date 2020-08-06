const FONT_LOCATION: usize = 0x50;
const MEMORY_SIZE: usize = 4096;
const DISPLAY_BUFFER_SIZE: usize = 512;
const STACK_START: usize = MEMORY_SIZE - DISPLAY_BUFFER_SIZE;

// mod memory;
mod utils;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    registers: RegisterBank,
    pc: u16,
    sp: u16,
}

struct RegisterBank {
    Vx: [u8; 16],
    I: u16,
    delay: u8,
    sound: u8,
}

impl Default for RegisterBank {
    fn default() -> Self {
        RegisterBank {
            Vx: [0; 16],
            I: 0,
            delay: 0,
            sound: 0,
        }
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            memory: [0; MEMORY_SIZE],
            registers: RegisterBank::default(),
            pc: 0,
            sp: STACK_START as u16,
        }
    }
}

#[wasm_bindgen]
impl Chip8 {
    pub fn new() -> Self {
        Chip8::default()
    }

    pub fn tick(&mut self) {
        let foo = self.fetch();
        let instruction = self.decode(foo);
        self.execute(instruction)
    }

    fn fetch(&mut self) -> u16 {
        self.pc += 2;
        0
    }

    fn decode(&self, instruction_code: u16) -> Instruction {
        match instruction_code {
            0 => Instruction::Add,
            _ => Instruction::Sub,
        }
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Add => println!("Adding!"),
            Instruction::Sub => println!("Subtracting!"),
            _ => unimplemented!()
        }
    }
}

enum Instruction {
    Add,
    Sub,
}


#[cfg(test)]
mod test {
    use super::Chip8;

    #[test]
    fn basic_test() {
        let chip8 = Chip8::new();
        chip8.tick()
    }

}
