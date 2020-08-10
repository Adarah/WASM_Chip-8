use js_sys::Error;
use wasm_bindgen::prelude::JsValue;

// might want to use a struct or hashmap of games instead?
pub const TETRIS: &'static [u8] = include_bytes!("TETRIS");
pub const BRIX: &'static [u8] = include_bytes!("BRIX");
pub const PONG: &'static [u8] = include_bytes!("PONG");
pub const SCTEST: &'static [u8] = include_bytes!("test_ROMs/SCTEST.ch8");
pub const BCTEST: &'static [u8] = include_bytes!("test_ROMs/BC_test.ch8");

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
            _ => return Err(Error::new("unknown game chosen").into()),
        };
        Ok(game)
    }
}

impl Default for Game {
    fn default() -> Self {
        Game { code: BCTEST, load_store_quirk: true, shift_quirk: true }
    }
}
