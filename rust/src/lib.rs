use web_sys::wasm_bindgen::prelude::wasm_bindgen;
use crate::model::universe::Universe;

mod model;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[wasm_bindgen]
pub fn generate_universe() -> Vec<usize> {
    Universe::generate(10, 10)
        .get_ids()
        .copied()
        .collect()
}