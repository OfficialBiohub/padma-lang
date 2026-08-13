#![allow(dead_code)]

include!("main.rs");

/// Stable text API used by future WASM, editor, and server integrations.
pub fn run_source(source: &str) -> Result<Vec<String>, String> {
    let (program, locale) = compile(source).map_err(|error| error.message.clone())?;
    let mut interpreter = Interpreter::new(locale);
    interpreter
        .run(&program)
        .map_err(|error| error.message.clone())?;
    Ok(interpreter.output)
}
