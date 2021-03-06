// //! Test suite for the Web and headless browsers.

// #![cfg(target_arch = "wasm32")]

// extern crate wasm_bindgen_test;
// use wasm_bindgen::prelude::JsValue;
// use wasm_bindgen_test::*;
// extern crate wasm_chip8;

// wasm_bindgen_test_configure!(run_in_browser);

// #[wasm_bindgen_test]
// fn pass() {
//     assert_eq!(1 + 1, 2);
// }

// use wasm_chip8::games::{BRIX, TETRIS};
// use wasm_chip8::Chip8;

// #[wasm_bindgen_test]
// fn loads_games() {
//     let mut chip8 = Chip8::new();
//     chip8.load_rom(JsValue::from_str("TETRIS")).unwrap();
//     let slice = unsafe { std::slice::from_raw_parts(chip8.memory_ptr(), 4096) };
//     assert_eq!(&slice[0x200..(0x200 + TETRIS.len())], TETRIS);

//     chip8.load_rom(JsValue::from_str("brix")).unwrap();
//     let slice = unsafe { std::slice::from_raw_parts(chip8.memory_ptr(), 4096) };
//     assert_eq!(&slice[0x200..(0x200 + BRIX.len())], BRIX);
// }

// #[wasm_bindgen_test]
// fn add_register_overflows_and_sets_flag() {
//     let mut chip8 = Chip8::new();
//     chip8.load_from_byte(0, 0xF0);
//     chip8.load_from_byte(1, 0x0F);
//     chip8.add_registers(0, 1);
//     assert_eq!(chip8.registers.Vx[0], 0xFF);
//     assert_eq!(chip8.registers.Vx[0xF], 0);

//     chip8.add_registers(0, 1);

//     assert_eq!(chip8.registers.Vx[0], 0x0E);
//     assert_eq!(chip8.registers.Vx[0xF], 1);
// }
