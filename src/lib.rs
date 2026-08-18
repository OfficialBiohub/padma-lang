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

/// Stable JSON diagnostic API for editor and language-server integrations.
/// The schema matches `padma check --json` exactly.
pub fn check_source_json(path: &str, source: &str) -> String {
    check_json(path, source)
}

/// Stable source-layout API for editor formatting integrations.
pub fn format_source_text(source: &str) -> String {
    format_source(source)
}
