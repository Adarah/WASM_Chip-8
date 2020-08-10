#![feature(slice_fill)]

#![allow(non_snake_case)]
pub mod games;
mod utils;

use games::Game;
use js_sys::Error;
use rand::{thread_rng, Rng};
use wasm_bindgen::prelude::*;
extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// mod memory;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const FONT_LOCATION: usize = 0x50;
const MEMORY_SIZE: usize = 4096;
const DISPLAY_BUFFER_SIZE: usize = 512;
const NEW_FRAME_START: usize = MEMORY_SIZE - DISPLAY_BUFFER_SIZE / 2;
const STACK_START: usize = MEMORY_SIZE - DISPLAY_BUFFER_SIZE;

#[wasm_bindgen]
pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    registers: RegisterBank,
    pc: usize,
    sp: usize,
    keypad: [bool; 16],
    game: Game,
}

struct RegisterBank {
    Vx: [u8; 16],
    I: usize,
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
        let mut memory = [0; MEMORY_SIZE];
        let fonts = [
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
        memory[FONT_LOCATION..(FONT_LOCATION + fonts.len())].copy_from_slice(&fonts);
        Chip8 {
            memory,
            registers: RegisterBank::default(),
            pc: 0x200,
            sp: STACK_START,
            keypad: [false; 16],
            game: Game::default(),
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

    pub fn display_buffer_ptr(&self) -> *const u8 {
        // log!("{:?}", &self.memory[STACK_START..NEW_FRAME_START]);
        self.memory[STACK_START..NEW_FRAME_START].as_ptr()
    }

    pub fn display_buffer_size(&self) -> usize {
        // The display buffer js should use is only half the size of the real display buffer.
        // The bottom half of the display buffer is what should be shown on screen,
        // the top half is what the game state is actually like. We use two representations to
        // reduce the flicking on the screen by ORing the previous frame with the current frame
        DISPLAY_BUFFER_SIZE / 2
    }

    pub fn press_key(&mut self, key: JsValue) -> Result<(), JsValue> {
        let idx = match key.as_f64() {
            Some(idx) => idx.round() as usize,
            None => return Err(Error::new("Could not parse key as f64").into()),
        };
        self.keypad[idx as usize] = true;
        Ok(())
    }

    pub fn release_key(&mut self, key: JsValue) -> Result<(), JsValue> {
        // this and the press_key function could be one function with 2 modes of operation,
        // but this feels like a better API.
        let idx = match key.as_f64() {
            Some(idx) => idx.round() as usize,
            None => return Err(Error::new("Could not parse key as f64").into()),
        };
        self.keypad[idx as usize] = false;
        Ok(())
    }

    pub fn load_rom(&mut self, title: JsValue) -> Result<(), JsValue> {
        match title.as_string() {
            None => Err(Error::new("Could not parse title as string").into()),
            Some(title) => {
                let game = games::Game::new(&title)?;

                for (mem, bytes) in self.memory.iter_mut().skip(0x200).zip(game.code) {
                    *mem = *bytes;
                }
                self.game = game;
                Ok(())
            }
        }
    }

    pub fn tick(&mut self) {
        let machine_code = self.fetch();
        self.decode_and_execute(machine_code)
    }

    pub fn decrement_timers(&mut self) {
        self.registers.delay = self.registers.delay.saturating_sub(1);
        self.registers.sound = self.registers.sound.saturating_sub(1);
    }

    fn fetch(&mut self) -> u16 {
        let machine_code = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        self.pc += 2;
        machine_code
    }

    fn decode_and_execute(&mut self, instruction_code: u16) {
        log!("instruction_code: {:04X?}", instruction_code);
        let nibbles = (
            (instruction_code >> 12 & 0x000F),
            (instruction_code >> 8 & 0x000F) as usize,
            (instruction_code >> 4 & 0x000F) as usize,
            (instruction_code & 0x000F) as u8,
        );
        let vx = self.registers.Vx[nibbles.1 as usize];
        let vy = self.registers.Vx[nibbles.2 as usize];
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
            (0x6, x, _, _) => self.load_from_byte(x, byte),
            (0x7, x, _, _) => self.add_byte(x, byte),
            (0x8, x, _, 0) => self.load_from_register(x, vy),
            (0x8, x, _, 1) => self.or(x, vy),
            (0x8, x, _, 2) => self.and(x, vy),
            (0x8, x, _, 3) => self.xor(x, vy),
            (0x8, x, _, 4) => self.add_registers(x, vy),
            (0x8, x, _, 5) => self.sub_vy_from_vx(x, vy),
            (0x8, x, y, 6) => self.shift_right(x, y), // logical right shift, not arithmetic
            (0x8, x, _, 7) => self.sub_vx_from_vy(x, vy),
            (0x8, x, y, 0xE) => self.shift_left(x, y),
            (0x9, _, _, 0) => self.skip_next_if_not_equal_to_register(vx, vy),
            (0xA, _, _, _) => self.set_I(triple),
            (0xB, _, _, _) => self.jump_relative(triple),
            (0xC, x, _, _) => self.set_random_number(x, byte),
            (0xD, _, _, n) => self.draw(vx, vy, n),
            (0xE, _, 9, 0xE) => self.skip_if_key_is_pressed(vx),
            (0xE, _, 0xA, 1) => self.skip_if_key_is_not_pressed(vx),
            (0xF, x, 0, 7) => self.load_from_delay_timer(x),
            (0xF, _, 0, 0xA) => self.block_until_key_is_pressed(vx),
            (0xF, _, 1, 5) => self.set_delay_timer(vx),
            (0xF, _, 1, 8) => self.set_sound_timer(vx),
            (0xF, _, 1, 0xE) => self.increment_i(vx),
            (0xF, _, 2, 9) => self.load_font_location_in_I(vx),
            (0xF, _, 3, 3) => self.store_bcd(vx),
            (0xF, x, 5, 5) => self.bulk_store(x),
            (0xF, x, 6, 5) => self.bulk_load(x),
            (0xF, _, 7, 5) => self.nop(),
            (0xF, _, 8, 5) => self.nop(),
            _ => panic!("Unknowin instruction encountered!"),
            // _ => Err(Error::new("Unimplemented instruction encountered!").into()),
        }
    }

    fn set_sp(&mut self, offset: isize) {
        log!("stack pointer: {}", self.sp);
        log!("offset: {}", offset);
        self.sp = (self.sp as isize + offset) as usize;
        log!("set stack pointer to: {}", self.sp);
        // if STACK_START - 12 * 2 >= self.sp {
        //     return Err(Error::new("StackOverflow: Stack exceeded maximum size").into());
        // }
        // if self.sp >= STACK_START {
        //     return Err(Error::new("StackUnderflow: Stack is invading frame buffer").into());
        // }
    }

    fn clear_display(&mut self) {
        log!("Clearing display");
        let (old_frame, current_frame) = self.memory.split_at_mut(NEW_FRAME_START);
        current_frame.fill(0);
        old_frame[STACK_START..NEW_FRAME_START].copy_from_slice(current_frame);
    }

    fn return_from_subroutine(&mut self) {
        self.pc = ((self.memory[self.sp] as usize) << 8) | (self.memory[self.sp + 1] as usize);
        log!("Returning from subroutine to: {}", self.pc);
        self.set_sp(2);
    }

    fn jump(&mut self, address: usize) {
        // log!("Jumping to: {:04X}", address);
        self.pc = address;
    }

    fn call_subroutine(&mut self, address: usize) {
        log!("Calling subroutine: {:04X}", address);
        self.set_sp(-2);
        self.memory[self.sp] = ((self.pc & 0xFF00) >> 8) as u8;
        self.memory[self.sp + 1] = (self.pc & 0x00FF) as u8;
        self.pc = address;
    }

    fn skip_next_if_equal_to_byte(&mut self, vx: u8, byte: u8) {
        log!("Skip next if vx == byte: ({}, {})", vx, byte);
        if vx == byte {
            self.pc += 2;
        }
    }

    fn skip_next_if_not_equal_to_byte(&mut self, vx: u8, byte: u8) {
        log!("Skip next if vx != byte: ({}, {})", vx, byte);
        if vx != byte {
            self.pc += 2;
        }
    }

    fn skip_next_if_equal_to_register(&mut self, vx: u8, vy: u8) {
        log!("Skip next if vx == vy: ({}, {})", vx, vy);
        if vx == vy {
            self.pc += 2;
        }
    }

    fn load_from_byte(&mut self, x: usize, byte: u8) {
        log!("Loading V{:X} with {}", x, byte);
        self.registers.Vx[x] = byte;
    }

    fn add_byte(&mut self, x: usize, byte: u8) {
        log!("Adding V{:X} with {}", x, byte);
        self.registers.Vx[x] = self.registers.Vx[x].wrapping_add(byte);
    }

    fn load_from_register(&mut self, x: usize, vy: u8) {
        log!("Load V{:X} with {}", x, vy);
        self.registers.Vx[x] = vy;
    }

    fn or(&mut self, x: usize, vy: u8) {
        log!("ORing V{:X} with {}", x, vy);
        self.registers.Vx[x] |= vy;
    }

    fn and(&mut self, x: usize, vy: u8) {
        log!("ANDing V{:X} with {}", x, vy);
        self.registers.Vx[x] &= vy;
    }

    fn xor(&mut self, x: usize, vy: u8) {
        log!("XORing V{:X} with {}", x, vy);
        self.registers.Vx[x] ^= vy;
    }

    fn add_registers(&mut self, x: usize, vy: u8) {
        log!("Adding V{:X} with {}", x, vy);
        let (result, overflow) = self.registers.Vx[x].overflowing_add(vy);
        self.registers.Vx[0xF] = if overflow {1} else {0};
        self.registers.Vx[x] = result;
    }

    fn sub_vy_from_vx(&mut self, x: usize, vy: u8) {
        log!("Subtracting {} from V{:X}", vy, x);
        let (result, overflow) = self.registers.Vx[x].overflowing_sub(vy);
        self.registers.Vx[0xF] = if overflow { 1 } else { 0 };
        self.registers.Vx[x] = result;
    }

    fn shift_right(&mut self, x: usize, y: usize) {
        log!("Right shifting V{:X}", x);
        if self.game.shift_quirk {
            self.registers.Vx[0xF] = self.registers.Vx[x] & 1;
            self.registers.Vx[x] = self.registers.Vx[x] >> 1;
        } else {
            self.registers.Vx[0xF] = self.registers.Vx[y] & 1;
            self.registers.Vx[x] = self.registers.Vx[y] >> 1;
        }
    }

    fn sub_vx_from_vy(&mut self, x: usize, vy: u8) {
        log!("Subbing V{:X} from {}", x, vy);
        let (result, overflow) = vy.overflowing_sub(self.registers.Vx[x]);
        self.registers.Vx[0xF] = if overflow { 1 } else { 0 };
        self.registers.Vx[x] = result;
    }

    fn shift_left(&mut self, x: usize, y: usize) {
        log!("Left shifting V{:X}", x);
        if self.game.shift_quirk {
            self.registers.Vx[0xF] = (self.registers.Vx[x] & 0b10000000) >> 7;
            self.registers.Vx[x] = self.registers.Vx[x] << 1;
        } else {
            self.registers.Vx[0xF] = (self.registers.Vx[y] & 0b10000000) >> 7;
            self.registers.Vx[x] = self.registers.Vx[y] << 1;
        }
    }

    fn skip_next_if_not_equal_to_register(&mut self, vx: u8, vy: u8) {
        // log!("Skip next if vx != vy ({}, {})", vx, vy);
        if vx != vy {
            self.pc += 2;
        }
    }

    fn set_I(&mut self, address: usize) {
        self.registers.I = address;
        log!("Setting I: {:04X}", self.registers.I);
    }

    fn jump_relative(&mut self, address: usize) {
        self.pc = address + (self.registers.Vx[0] as usize);
        log!("Jumping relative to: {:04X}", self.pc);
    }

    fn set_random_number(&mut self, x: usize, byte: u8) {
        let mut rng = thread_rng();
        let random_num: u8 = rng.gen();
        log!("chose random num: {}", random_num);
        self.registers.Vx[x] = random_num & byte;
        log!("Setting random number: {:02X}", self.registers.Vx[x]);
    }

    fn draw(&mut self, vx: u8, vy: u8, n: u8) {
        log!("Draw args: vx = {}, vy = {}, n = {}", vx, vy, n);
        let (rest, current_frame) = self.memory.split_at_mut(NEW_FRAME_START);
        let (rest, smoothed_frame) = rest.split_at_mut(STACK_START);
        smoothed_frame.copy_from_slice(current_frame);
        self.registers.Vx[0xF] = 0;

        let sprite = &rest[self.registers.I..(self.registers.I + n as usize)];
        // log!("sprite: {:?}", sprite);
        for y in 0..n {
            let sprite_byte = sprite[y as usize];
            for x in 0..8 {
                let row = ((vx + x) % 64) as usize;
                let col = ((vy + y) % 32) as usize;
                let byte = (col * 64 + row) / 8;
                // log!("sprite_byte: {:08b}", sprite_byte);
                // log!("screen_byte: {:08b}", current_frame[byte]);
                let screen_mask: u8 = 0b1000_0000 >> (col * 64 + row) % 8;
                let sprite_bit = sprite_byte & (0b1000_0000 >> x);
                // log!("screen mask: {:08b}", screen_mask);

                // log!("sprite bit: {:08b}", sprite_bit);

                // log!("current_frame_byte: {:08b}", current_frame[byte]);
                if sprite_bit != 0 {
                    if current_frame[byte] & screen_mask != 0 {
                        self.registers.Vx[0xF] = 1;
                    }
                    current_frame[byte] ^= screen_mask
                }

                // current_frame[byte] ^=
                //     (current_frame[byte] & screen_mask) & sprite_byte & (0b1000_0000 >> x);
                // log!("result: {:08b}", current_frame[byte]);
            }
        }

        for (smooth, raw) in smoothed_frame.iter_mut().zip(current_frame.iter()) {
            *smooth |= raw;
        }
    }

    fn skip_if_key_is_pressed(&mut self, vx: u8) {
        log!("Skip next if key is pressed: {:X}", vx);
        if self.keypad[vx as usize] {
            self.pc += 2;
        }
    }

    fn skip_if_key_is_not_pressed(&mut self, vx: u8) {
        log!("Skip next if key is not pressed: {:X}", vx);
        if !self.keypad[vx as usize] {
            self.pc += 2;
        }
    }

    fn load_from_delay_timer(&mut self, x: usize) {
        log!(
            "Loading V{:X} with delay timer: {}",
            x,
            self.registers.delay
        );
        self.registers.Vx[x] = self.registers.delay;
    }

    fn block_until_key_is_pressed(&mut self, vx: u8) {
        log!("Block until key is pressed: {}", vx);
        if !self.keypad[vx as usize] {
            self.pc -= 2;
        }
    }

    fn set_delay_timer(&mut self, vx: u8) {
        log!("Set delay timer: {}", vx);
        self.registers.delay = vx;
    }

    fn set_sound_timer(&mut self, vx: u8) {
        log!("Set sound timer: {}", vx);
        self.registers.sound = vx;
    }

    fn increment_i(&mut self, vx: u8) {
        log!("Increment I: {}", vx);
        self.registers.I += vx as usize;
    }

    fn load_font_location_in_I(&mut self, vx: u8) {
        self.registers.I = FONT_LOCATION + (vx as usize) * 5;
        log!("Set I to font location of {}: {}", vx, self.registers.I);
    }

    fn store_bcd(&mut self, vx: u8) {
        log!("Store BCD: {}", vx);
        let hundreds = vx / 100;
        let tens = (vx % 100) / 10;
        let ones = vx % 10;
        self.memory[self.registers.I] = hundreds;
        self.memory[self.registers.I + 1] = tens;
        self.memory[self.registers.I + 2] = ones;
    }

    fn bulk_store(&mut self, x: usize) {
        log!("Bulk store from V0 to V{:X}", x);
        self.memory[self.registers.I..self.registers.I + x + 1]
            .copy_from_slice(&self.registers.Vx[0..x + 1]);
        if !self.game.load_store_quirk {
            self.registers.I = self.registers.I + x + 1;
        }
    }

    fn bulk_load(&mut self, x: usize) {
        log!("Bulk load into V0 to V{:X}", x);
        self.registers.Vx[0..x + 1]
            .copy_from_slice(&self.memory[self.registers.I..self.registers.I + x + 1]);
        log!("registers: {:?}", self.registers.Vx);
        log!(
            "memory region: {:?}",
            &self.memory[self.registers.I..self.registers.I + x + 1]
        );
        if !self.game.load_store_quirk {
            self.registers.I = self.registers.I + x + 1;
        }
    }

    fn nop(&mut self) {
        self.pc += 54;
    }
}

#[cfg(wasm_bindgen_test)]
mod tests {
    use super::*;

}

mod idk {

    #![cfg(target_arch = "wasm32")]

    use wasm_bindgen_test;
    use wasm_bindgen::prelude::JsValue;
    use wasm_bindgen_test::*;
    use super::*;
    use super::games::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn loads_games() {
        let mut chip8 = Chip8::new();
        chip8.load_rom(JsValue::from_str("TETRIS")).unwrap();
        let slice = unsafe { std::slice::from_raw_parts(chip8.memory_ptr(), 4096) };
        assert_eq!(&slice[0x200..(0x200 + TETRIS.len())], TETRIS);

        chip8.load_rom(JsValue::from_str("brix")).unwrap();
        let slice = unsafe { std::slice::from_raw_parts(chip8.memory_ptr(), 4096) };
        assert_eq!(&slice[0x200..(0x200 + BRIX.len())], BRIX);
    }

    #[wasm_bindgen_test]
    fn add_register_overflows_and_sets_flag() {
        let mut chip8 = Chip8::new();
        chip8.load_from_byte(0, 0xF0);
        chip8.add_registers(0, 0x0F);
        assert_eq!(chip8.registers.Vx[0], 0xFF);
        assert_eq!(chip8.registers.Vx[0xF], 0);

        chip8.add_registers(0, 0x0F);

        assert_eq!(chip8.registers.Vx[0], 0x0E);
        assert_eq!(chip8.registers.Vx[0xF], 1);
    }

    #[wasm_bindgen_test]
    fn draws_correctly() {
        let mut chip8 = Chip8::new();
        chip8.load_rom(JsValue::from_str("bctest"));
        chip8.set_I(000);
        let e: [u8; 5] = [0xF0, 0x80, 0xF0, 0x80, 0xF0]; // E
        chip8.memory[0..5].copy_from_slice(&e);
        chip8.draw(0, 0, 5);
        // log!("{:X?}", &chip8.memory[NEW_FRAME_START..]);
        for i in 0..5 {
            assert_eq!(chip8.memory[NEW_FRAME_START + 8 *i], e[i]);
        }
        assert_eq!(chip8.registers.Vx[0xF], 0);

        chip8.draw(0, 0, 5);
        for i in 0..5 {
            assert_eq!(chip8.memory[NEW_FRAME_START + 8 * i], 0);
        }
        assert_eq!(chip8.registers.Vx[0xF], 1);
    }
    #[wasm_bindgen_test]
    fn return_reverts_call() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 0x24;
        chip8.memory[0x201] = 0x00;
        chip8.tick();
        chip8.memory[0x400] = 0x00;
        chip8.memory[0x401] = 0xEE;
        chip8.tick();
        assert_eq!(chip8.pc, 0x202);
        assert_eq!(chip8.sp, STACK_START);
    }

    #[wasm_bindgen_test]
    fn check_sub_vy_from_vx() {
        let mut chip8 = Chip8::new();
        chip8.registers.Vx[0] = 2;
        chip8.registers.Vx[1] = 1;
        chip8.memory[0x200] = 0x80;
        chip8.memory[0x201] = 0x15;
        chip8.tick();
        assert_eq!(chip8.registers.Vx[0], 1);
        assert_eq!(chip8.registers.Vx[0xF], 0);
        chip8.pc = 0x200;
        chip8.tick();
        assert_eq!(chip8.registers.Vx[0], 0);
        assert_eq!(chip8.registers.Vx[0xF], 0);
        chip8.pc = 0x200;
        chip8.tick();
        assert_eq!(chip8.registers.Vx[0], 0xFF);
        assert_eq!(chip8.registers.Vx[0xF], 1);
    }

    #[wasm_bindgen_test]
    fn check_sub_vx_from_vy() {
        let mut chip8 = Chip8::new();
        chip8.registers.Vx[0] = 1;
        chip8.registers.Vx[1] = 2;
        chip8.memory[0x200] = 0x80;
        chip8.memory[0x201] = 0x17;
        chip8.tick();
        assert_eq!(chip8.registers.Vx[0], 1);
        assert_eq!(chip8.registers.Vx[0xF], 0);
        chip8.registers.Vx[0] = 3;
        chip8.pc = 0x200;
        chip8.tick();
        assert_eq!(chip8.registers.Vx[0], 0xFF);
        assert_eq!(chip8.registers.Vx[0xF], 1);
    }

}
