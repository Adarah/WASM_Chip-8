use js_sys::Error;
use wasm_bindgen::prelude::JsValue;

pub const TETRIS: &'static [u8] = include_bytes!("TETRIS");
// pub const BRIX: &'static [u8] = include_bytes!("BRIX");

pub fn parse_title(title: JsValue) -> Result<&'static [u8], JsValue> {
    if let Some(title) = title.as_string() {
        let bytes = match title.to_lowercase().as_str() {
            "tetris" => TETRIS,
            // "brix" => BRIX,
            _ => return Err(Error::new("unknown game chosen").into()),
        };
        return Ok(&bytes);
    }
    Err(Error::new("Could not parse title as string").into())
}
