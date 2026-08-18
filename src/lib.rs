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

/// Stable non-executing declaration API for editor integrations.
/// Positions are one-based Padma source coordinates and callers must convert them
/// to their protocol-specific encoding (such as LSP UTF-16) at the boundary.
pub fn local_declarations_json(source: &str) -> Result<serde_json::Value, String> {
    let (program, _) = parse_source_recovering(source).map_err(|errors| {
        errors
            .first()
            .map(|error| error.message.clone())
            .unwrap_or_else(|| "Cannot analyze source.".to_string())
    })?;

    fn collect(statements: &[Stmt], depth: usize, output: &mut Vec<serde_json::Value>) {
        for statement in statements {
            match statement {
                Stmt::Let {
                    name,
                    name_position,
                    ..
                } => output.push(serde_json::json!({
                    "name": name,
                    "kind": "variable",
                    "scopeDepth": depth,
                    "line": name_position.line,
                    "column": name_position.column
                })),
                Stmt::For {
                    name,
                    name_position,
                    body,
                    ..
                } => {
                    output.push(serde_json::json!({
                        "name": name,
                        "kind": "loop-variable",
                        "scopeDepth": depth + 1,
                        "line": name_position.line,
                        "column": name_position.column
                    }));
                    collect(body, depth + 1, output);
                }
                Stmt::Function {
                    name,
                    name_position,
                    body,
                    ..
                } => {
                    output.push(serde_json::json!({
                        "name": name,
                        "kind": "function",
                        "scopeDepth": depth,
                        "line": name_position.line,
                        "column": name_position.column
                    }));
                    collect(body, depth + 1, output);
                }
                Stmt::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    collect(then_branch, depth + 1, output);
                    collect(else_branch, depth + 1, output);
                }
                Stmt::While { body, .. } => collect(body, depth + 1, output),
                Stmt::Export(inner) => collect(std::slice::from_ref(inner), depth, output),
                _ => {}
            }
        }
    }

    let mut declarations = Vec::new();
    collect(&program, 0, &mut declarations);
    Ok(serde_json::Value::Array(declarations))
}
