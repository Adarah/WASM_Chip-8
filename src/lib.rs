const FONT_LOCATION: usize = 0x50;
const MEMORY_SIZE: usize = 4096;
const DISPLAY_BUFFER_SIZE: usize = 512;
const STACK_START: usize = MEMORY_SIZE - DISPLAY_BUFFER_SIZE;

// mod memory;
pub mod games;
mod utils;

use js_sys::Error;
use wasm_bindgen::prelude::*;
use rand::{thread_rng, Rng};

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
    pc: usize,
    sp: usize,
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
            sp: STACK_START,
        }
    }
}

#[wasm_bindgen]
impl Chip8 {
    pub fn new() -> Self {
        utils::set_panic_hook();
        Chip8::default()
    }

    pub fn memory_ptr(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    pub fn load_rom(&mut self, game: JsValue) -> Result<(), JsValue> {
        let game_data = games::parse_title(game)?;
        for (mem, bytes) in self.memory.iter_mut().skip(0x200).zip(game_data) {
            *mem = *bytes;
        }
        self.sp = STACK_START;
        self.pc = 0x200;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), JsValue> {
        let machine_code = self.fetch();
        self.decode_and_execute(machine_code).into()
    }

    fn fetch(&mut self) -> u16 {
        let machine_code = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        self.pc += 2;
        machine_code
    }

    fn decode_and_execute(&mut self, instruction_code: u16) -> Result<(), JsValue> {
        let nibbles = (
            instruction_code >> 12 & 0x000F,
            instruction_code >> 8 & 0x000F,
            instruction_code >> 4 & 0x000F,
            instruction_code & 0x000F,
        );
        let vx = nibbles.1 as usize;
        let vy = nibbles.2 as usize;
        let byte = (instruction_code & 0x00FF) as u8;
        let triple = (instruction_code & 0x0FFF) as usize;
        match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.clear_display(),
            (0x0, 0x0, 0xE, 0xE) => self.return_from_subroutine(),
            (0x1, _, _, _) => self.jump(triple),
            (0x2, _, _, _) => self.call_subroutine(triple),
            (0x3, _, _, _) => self.skip_next_if_equal_to_byte(vx, byte),
            (0x4, _, _, _) => self.skip_next_if_not_equal_to_byte(vx, byte),
            (0x5, _, _, 0) => self.skip_next_if_equal_to_register(vx, vy),
            (0x6, _, _, _) => self.load_from_byte(vx, byte),
            (0x7, _, _, _) => self.add_byte(vx, byte),
            (0x8, _, _, 0) => self.load_from_register(vx, vy),
            (0x8, _, _, 1) => self.or(vx, vy),
            (0x8, _, _, 2) => self.and(vx, vy),
            (0x8, _, _, 3) => self.xor(vx, vy),
            (0x8, _, _, 4) => self.add_registers(vx, vy),
            (0x8, _, _, 5) => self.sub_vy_from_vx(vx, vy),
            (0x8, _, _, 6) => self.shift_right(vx), // logical right shift, not arithmetic
            (0x8, _, _, 7) => self.sub_vx_from_vy(vx, vy),
            (0x8, _, _, 0xE) => self.shift_left(vx),
            (0x9, _, _, 0) => self.skip_next_if_not_equal_to_register(vx, vy),
            (0xA, _, _, _) => self.set_I(triple),
            (0xB, _, _, _) => self.jump_relative(triple),
            (0xC, _, _, _) => self.set_random_number(vx, byte),
            // (0xD, _, _, n) => self.draw(vx, vy, n),
            // (0xE, _, 9, 0xE) => self.skip_if_key_is_pressed(vx),
            // (0xE, _, 0xA, 1) => self.skip_if_key_is_not_pressed(vx),
            // (0xF, _, 0, 7) => self.load_from_delay_timer(vx),
            // (0xF, _, 0, 0xA) => self.block_until_key_is_pressed(vx),
            // (0xF, _, 1, 5) => self.set_delay_timer(vx),
            // (0xF, _, 1, 8) => self.set_sound_timer(vx),
            // (0xF, _, 1, 0xE) => self.increment_I(vx),
            // (0xF, _, 2, 9) => self.load_font_location_in_I(vx),
            // (0xF, _, 3, 3) => self.store_BCD(vx),
            // (0xF, _, 5, 5) => self.bulk_store(vx),
            // (0xF, _, 6, 5) => self.bulk_load(vx),
            _ => unimplemented!("Unknown instruction encountered!"),
        }
    }

    fn set_sp(&mut self, offset: isize) -> Result<(), JsValue> {
        self.sp = (self.sp as isize + offset) as usize;
        if !(STACK_START - 12*2 < self.sp && self.sp < STACK_START) {
            return Err(Error::new("StackOverflow: Stack exceeded maximum size").into());
        }
        Ok(())
    }

    fn set_pc(&mut self, address: usize) -> Result<(), JsValue> {
        self.sp = address;
        if self.sp % 2 != 0 {
            return Err(Error::new("AlignementError: Unaligned memory access attempted!").into());
        }
        Ok(())
    }

    fn clear_display(&mut self) -> Result<(), JsValue> {
        for i in (MEMORY_SIZE - DISPLAY_BUFFER_SIZE)..MEMORY_SIZE {
            self.memory[i] = 0;
        }
        Ok(())
    }

    fn return_from_subroutine(&mut self) -> Result<(), JsValue> {
        self.pc = (self.memory[self.sp] as usize) << 8 | self.memory[self.sp + 1] as usize;
        self.set_sp(2)?;
        Ok(())
    }

    fn jump(&mut self, address: usize) -> Result<(), JsValue> {
        self.set_pc(address)
    }

    fn call_subroutine(&mut self, address: usize) -> Result<(), JsValue> {
        self.set_sp(-2)?;
        self.memory[self.sp] = (address & 0xFF00) as u8;
        self.memory[self.sp + 1] = (address & 0x00FF) as u8;
        Ok(())
    }

    fn skip_next_if_equal_to_byte(&mut self, x: usize, byte: u8) -> Result<(), JsValue> {
        if self.registers.Vx[x] == byte {
            self.pc += 2;
        }
        Ok(())
    }

    fn skip_next_if_not_equal_to_byte(&mut self, x: usize, byte: u8) -> Result<(), JsValue> {
        if self.registers.Vx[x] != byte {
            self.pc += 2;
        }
        Ok(())
    }

    fn skip_next_if_equal_to_register(&mut self, x: usize, y: usize) -> Result<(), JsValue> {
        if self.registers.Vx[x] == self.registers.Vx[y] {
            self.pc += 2;
        }
        Ok(())
    }

    fn load_from_byte(&mut self, x: usize, byte: u8) -> Result<(), JsValue> {
        self.registers.Vx[x] = byte;
        Ok(())
    }

    fn add_byte(&mut self, x: usize, byte: u8) -> Result<(), JsValue> {
        self.registers.Vx[x] += byte;
        Ok(())
    }

    fn load_from_register(&mut self, x: usize, y: usize) -> Result<(), JsValue> {
        self.registers.Vx[x] = self.registers.Vx[y];
        Ok(())
    }

    fn or(&mut self, x: usize, y: usize) -> Result<(), JsValue> {
        self.registers.Vx[x] |= self.registers.Vx[y];
        Ok(())
    }

    fn and(&mut self, x: usize, y: usize) -> Result<(), JsValue> {
        self.registers.Vx[x] &= self.registers.Vx[y];
        Ok(())
    }

    fn xor(&mut self, x: usize, y: usize) -> Result<(), JsValue> {
        self.registers.Vx[x] ^= self.registers.Vx[y];
        Ok(())
    }

    fn add_registers(&mut self, x: usize, y: usize) -> Result<(), JsValue> {
        self.registers.Vx[0xF] = 0;

        self.registers.Vx[x] = self.registers.Vx[x]
            .checked_add(self.registers.Vx[y])
            .unwrap_or_else(|| {
                self.registers.Vx[0xF] = 1;
                self.registers.Vx[x].wrapping_add(self.registers.Vx[y])
            });
        Ok(())
    }

    fn sub_vy_from_vx(&mut self, x: usize, y:usize) -> Result<(), JsValue> {
        if self.registers.Vx[x] > self.registers.Vx[y] {
            self.registers.Vx[0xF] = 1;
        } else {
            self.registers.Vx[0xF] = 0;
        }

        self.registers.Vx[x] -= self.registers.Vx[y];
        Ok(())
    }

    fn shift_right(&mut self, x: usize) -> Result<(), JsValue> {
        self.registers.Vx[0xF] = self.registers.Vx[x] & 0b0001;
        self.registers.Vx[x] = self.registers.Vx[x] >> 1;
        Ok(())
    }

    fn sub_vx_from_vy(&mut self, x: usize, y:usize) -> Result<(), JsValue> {
        if self.registers.Vx[y] > self.registers.Vx[x] {
            self.registers.Vx[0xF] = 1;
        } else {
            self.registers.Vx[0xF] = 0;
        }

        self.registers.Vx[x] = self.registers.Vx[y] - self.registers.Vx[x];
        Ok(())
    }

    fn shift_left(&mut self, x: usize) -> Result<(), JsValue> {
        self.registers.Vx[0xF] = self.registers.Vx[x] & 0b1000;
        self.registers.Vx[x] = self.registers.Vx[x] << 1;
        Ok(())
    }

    fn skip_next_if_not_equal_to_register(&mut self, x: usize, y: usize) -> Result<(), JsValue> {
        if self.registers.Vx[x] != self.registers.Vx[y] {
            self.pc += 2;
        }
        Ok(())
    }

    fn set_I(&mut self, address: usize) -> Result<(), JsValue> {
        self.registers.I = address as u16;
        Ok(())
    }

    fn jump_relative(&mut self, address: usize) -> Result<(), JsValue> {
        self.set_pc(address + self.registers.Vx[0] as usize)
    }

    fn set_random_number(&mut self, x: usize, byte: u8) -> Result<(), JsValue> {
        let mut rng = thread_rng();
        let random_num: u8 = rng.gen();
        self.registers.Vx[x] = random_num & byte;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_register_overflows_and_sets_flag() {
        let mut chip8 = Chip8::new();
        chip8.load_from_byte(0, 0xF0);
        chip8.load_from_byte(1, 0x0F);
        chip8.add_registers(0, 1);
        assert_eq!(chip8.registers.Vx[0], 0xFF);
        assert_eq!(chip8.registers.Vx[0xF], 0);

        chip8.add_registers(0, 1);

        assert_eq!(chip8.registers.Vx[0], 0x0E);
        assert_eq!(chip8.registers.Vx[0xF], 1);
    }
}
