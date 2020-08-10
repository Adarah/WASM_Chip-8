use js_sys::Error;
use wasm_bindgen::prelude::JsValue;

// might want to use a struct or hashmap of games instead?
pub const TETRIS: &'static [u8] = include_bytes!("TETRIS");
pub const BRIX: &'static [u8] = include_bytes!("BRIX");
pub const PONG: &'static [u8] = include_bytes!("PONG");
pub const PONG2: &'static [u8] = include_bytes!("PONG2");
pub const INVADERS: &'static [u8] = include_bytes!("INVADERS");
pub const SCTEST: &'static [u8] = include_bytes!("test_ROMs/SCTEST.ch8");
pub const BCTEST: &'static [u8] = include_bytes!("test_ROMs/BC_test.ch8");
pub const C8TEST: &'static [u8] = include_bytes!("test_ROMs/c8_test.ch8");
pub const SAMPLE: &'static [u8] = include_bytes!("test_ROMs/sample.ch8");
pub const OPCODE_TEST: &'static [u8] = include_bytes!("test_ROMs/opcode_test.ch8");

pub struct Game {
    pub code: &'static [u8],
    pub load_store_quirk: bool,
    pub shift_quirk: bool,
}

impl Game {
    pub fn new(title: &str) -> Result<Self, JsValue> {
        let game = match title.to_lowercase().as_str() {
            "tetris" => Game {
                code: TETRIS,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "brix" => Game {
                code: BRIX,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "pong" => Game {
                code: PONG,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "pong2" => Game {
                code: PONG2,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "invaders" => Game {
                code: INVADERS,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "sctest" => Game {
                code: SCTEST,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "bctest" => Game {
                code: BCTEST,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "c8test" => Game {
                code: C8TEST,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "sample" => Game {
                code: SAMPLE,
                load_store_quirk: true,
                shift_quirk: true,
            },
            "opcode_test" => Game {
                code: OPCODE_TEST,
                load_store_quirk: true,
                shift_quirk: true,
            },
            _ => return Err(Error::new("unknown game chosen").into()),
        };
        Ok(game)
    }
}

impl Default for Game {
    fn default() -> Self {
        Game {
            code: BCTEST,
            load_store_quirk: true,
            shift_quirk: true,
        }
    }
}
