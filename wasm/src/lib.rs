use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_padma(source: &str) -> String {
    match padma_lang::run_source(source) {
        Ok(lines) => format!("OK\n{}", lines.join("\n")),
        Err(error) => format!("ERR\n{error}"),
    }
}
