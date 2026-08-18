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

/// Stable non-executing local binding report for editor navigation and rename.
/// It intentionally reports only same-document variable bindings; functions,
/// imports, exports, members, strings, and comments are never rename targets.
pub fn local_reference_bindings_json(source: &str) -> Result<serde_json::Value, String> {
    use std::collections::HashMap;

    let (program, _) = parse_source_recovering(source).map_err(|errors| {
        errors
            .first()
            .map(|error| error.message.clone())
            .unwrap_or_else(|| "Cannot analyze source.".to_string())
    })?;
    #[derive(Clone)]
    struct Binding {
        id: usize,
        name: String,
        position: Position,
        depth: usize,
    }
    let mut next_id = 1usize;
    let mut declarations = Vec::<Binding>::new();
    let mut references = Vec::<serde_json::Value>::new();

    fn visit_expr(
        expr: &Expr,
        scopes: &Vec<HashMap<String, Binding>>,
        references: &mut Vec<serde_json::Value>,
    ) {
        match expr {
            Expr::Variable(name, position) => {
                for scope in scopes.iter().rev() {
                    if let Some(binding) = scope.get(name) {
                        references.push(serde_json::json!({
                            "bindingId": binding.id, "name": name,
                            "line": position.line, "column": position.column
                        }));
                        break;
                    }
                }
            }
            Expr::Unary { right, .. } => visit_expr(right, scopes, references),
            Expr::Binary { left, right, .. } => {
                visit_expr(left, scopes, references);
                visit_expr(right, scopes, references);
            }
            Expr::Call { arguments, .. } => {
                for argument in arguments {
                    visit_expr(argument, scopes, references);
                }
            }
            Expr::Index { target, index, .. } => {
                visit_expr(target, scopes, references);
                visit_expr(index, scopes, references);
            }
            Expr::Slice {
                target, start, end, ..
            } => {
                visit_expr(target, scopes, references);
                if let Some(value) = start {
                    visit_expr(value, scopes, references);
                }
                if let Some(value) = end {
                    visit_expr(value, scopes, references);
                }
            }
            Expr::List(values) => {
                for value in values {
                    visit_expr(value, scopes, references);
                }
            }
            Expr::Map(values) => {
                for (key, value) in values {
                    visit_expr(key, scopes, references);
                    visit_expr(value, scopes, references);
                }
            }
            Expr::Literal(_, _) => {}
        }
    }
    fn visit_block(
        statements: &[Stmt],
        scopes: &mut Vec<HashMap<String, Binding>>,
        depth: usize,
        next_id: &mut usize,
        declarations: &mut Vec<Binding>,
        references: &mut Vec<serde_json::Value>,
    ) {
        for statement in statements {
            match statement {
                Stmt::Let {
                    name,
                    name_position,
                    value,
                } => {
                    visit_expr(value, scopes, references);
                    let binding = Binding {
                        id: *next_id,
                        name: name.clone(),
                        position: *name_position,
                        depth,
                    };
                    *next_id += 1;
                    declarations.push(binding.clone());
                    scopes.last_mut().unwrap().insert(name.clone(), binding);
                }
                Stmt::Assign {
                    name,
                    position,
                    value,
                } => {
                    visit_expr(value, scopes, references);
                    for scope in scopes.iter().rev() {
                        if let Some(binding) = scope.get(name) {
                            references.push(serde_json::json!({ "bindingId": binding.id, "name": name, "line": position.line, "column": position.column }));
                            break;
                        }
                    }
                }
                Stmt::Print { value } | Stmt::Expression { value } => {
                    visit_expr(value, scopes, references)
                }
                Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    visit_expr(condition, scopes, references);
                    scopes.push(HashMap::new());
                    visit_block(
                        then_branch,
                        scopes,
                        depth + 1,
                        next_id,
                        declarations,
                        references,
                    );
                    scopes.pop();
                    scopes.push(HashMap::new());
                    visit_block(
                        else_branch,
                        scopes,
                        depth + 1,
                        next_id,
                        declarations,
                        references,
                    );
                    scopes.pop();
                }
                Stmt::While {
                    condition, body, ..
                } => {
                    visit_expr(condition, scopes, references);
                    scopes.push(HashMap::new());
                    visit_block(body, scopes, depth + 1, next_id, declarations, references);
                    scopes.pop();
                }
                Stmt::For {
                    name,
                    name_position,
                    collection,
                    body,
                    ..
                } => {
                    visit_expr(collection, scopes, references);
                    scopes.push(HashMap::new());
                    let binding = Binding {
                        id: *next_id,
                        name: name.clone(),
                        position: *name_position,
                        depth: depth + 1,
                    };
                    *next_id += 1;
                    declarations.push(binding.clone());
                    scopes.last_mut().unwrap().insert(name.clone(), binding);
                    visit_block(body, scopes, depth + 1, next_id, declarations, references);
                    scopes.pop();
                }
                Stmt::Function { .. } | Stmt::Import { .. } => {}
                Stmt::Return { value } => {
                    if let Some(value) = value {
                        visit_expr(value, scopes, references);
                    }
                }
                // Exported declarations are intentionally excluded. The LSP rename
                // contract is local-only until public API and cross-file analysis
                // can prove every downstream reference is safe to update.
                Stmt::Export(_) => {}
            }
        }
    }
    let mut scopes = vec![HashMap::new()];
    visit_block(
        &program,
        &mut scopes,
        0,
        &mut next_id,
        &mut declarations,
        &mut references,
    );
    Ok(serde_json::json!({
        "declarations": declarations.into_iter().map(|item| serde_json::json!({ "bindingId": item.id, "name": item.name, "line": item.position.line, "column": item.position.column, "scopeDepth": item.depth })).collect::<Vec<_>>(),
        "references": references
    }))
}
