use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};

use serde_json::{json, Value};

struct Server {
    documents: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentSymbol {
    name: String,
    kind: &'static str,
    line: usize,
    character: usize,
    scope_depth: usize,
}

impl Server {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    fn publish_diagnostics(&self, uri: &str, output: &mut impl Write) -> io::Result<()> {
        let source = self.documents.get(uri).map(String::as_str).unwrap_or("");
        let report: Value = serde_json::from_str(&padma_lang::check_source_json(uri, source))
            .unwrap_or_else(|_| json!({ "diagnostics": [] }));
        let diagnostics = report["diagnostics"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .map(|item| lsp_diagnostic(source, item))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        write_message(
            output,
            &json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": { "uri": uri, "diagnostics": diagnostics }
            }),
        )
    }

    fn format_document(&self, uri: &str) -> Value {
        let source = self.documents.get(uri).map(String::as_str).unwrap_or("");
        json!([{
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": end_position(source)
            },
            "newText": padma_lang::format_source_text(source)
        }])
    }

    fn hover(&self, uri: &str, position: &Value) -> Value {
        let source = self.documents.get(uri).map(String::as_str).unwrap_or("");
        let word = word_at(source, position).unwrap_or_default();
        let message = match word.as_str() {
            "let" | "ধরি" => Some("Padma variable declaration."),
            "function" | "ফাংশন" => Some("Padma function declaration."),
            "if" | "যদি" => Some("Padma conditional statement."),
            "while" | "যতক্ষণ" => Some("Padma bounded loop statement."),
            "for" | "প্রতি" => Some("Padma collection iteration statement."),
            "input" => Some("Read one line of user input."),
            "range" | "পরিসর" => Some("Create a bounded integer range."),
            _ => None,
        };
        message.map_or(Value::Null, |value| json!({ "contents": { "kind": "markdown", "value": value } }))
    }

    fn definition(&self, uri: &str, position: &Value) -> Value {
        let source = self.documents.get(uri).map(String::as_str).unwrap_or("");
        let word = word_at(source, position).unwrap_or_default();
        let line = position["line"].as_u64().unwrap_or(0) as usize;
        let scope_depth = scope_depth_at(source, line);
        document_symbols(source)
            .into_iter()
            .filter(|symbol| symbol.name == word && symbol.line <= line && symbol.scope_depth <= scope_depth)
            .max_by_key(|symbol| (symbol.scope_depth, symbol.line))
            .map_or(Value::Null, |symbol| json!({
                "uri": uri,
                "range": {
                    "start": { "line": symbol.line, "character": symbol.character },
                    "end": { "line": symbol.line, "character": symbol.character + symbol.name.encode_utf16().count() }
                }
            }))
    }

    fn completion(&self, uri: &str, position: &Value) -> Value {
        let source = self.documents.get(uri).map(String::as_str).unwrap_or("");
        let line = position["line"].as_u64().unwrap_or(usize::MAX as u64) as usize;
        let scope_depth = scope_depth_at(source, line);
        let mut items = completion_items().as_array().cloned().unwrap_or_default();
        for symbol in document_symbols(source)
            .into_iter()
            .filter(|symbol| symbol.line <= line && symbol.scope_depth <= scope_depth)
        {
            if !items.iter().any(|item| item["label"] == symbol.name) {
                items.push(json!({ "label": symbol.name, "detail": symbol.kind }));
            }
        }
        Value::Array(items)
    }

    fn rename_binding(&self, uri: &str, position: &Value) -> Option<(u64, Value)> {
        let source = self.documents.get(uri).map(String::as_str).unwrap_or("");
        let range = word_range_at(source, position)?;
        let report = padma_lang::local_reference_bindings_json(source).ok()?;
        let line = range["start"]["line"].as_u64()? as usize;
        let character = range["start"]["character"].as_u64()? as usize;
        for item in report["declarations"].as_array()?.iter().chain(report["references"].as_array()?.iter()) {
            let source_line = item["line"].as_u64()? as usize;
            let source_column = item["column"].as_u64()? as usize;
            if source_line.saturating_sub(1) == line && utf16_column(source, source_line, source_column) == character {
                return Some((item["bindingId"].as_u64()?, range));
            }
        }
        None
    }

    fn prepare_rename(&self, uri: &str, position: &Value) -> Value {
        self.rename_binding(uri, position).map_or(Value::Null, |(_, range)| range)
    }

    fn rename(&self, uri: &str, position: &Value, new_name: &str) -> Value {
        if !is_valid_identifier(new_name) {
            return Value::Null;
        }
        let Some((binding_id, _)) = self.rename_binding(uri, position) else {
            return Value::Null;
        };
        let source = self.documents.get(uri).map(String::as_str).unwrap_or("");
        let Ok(report) = padma_lang::local_reference_bindings_json(source) else {
            return Value::Null;
        };
        let target_depth = report["declarations"]
            .as_array()
            .and_then(|items| items.iter().find(|item| item["bindingId"].as_u64() == Some(binding_id)))
            .and_then(|item| item["scopeDepth"].as_u64());
        if report["declarations"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|item| {
                item["bindingId"].as_u64() != Some(binding_id)
                    && item["name"].as_str() == Some(new_name)
                    && item["scopeDepth"].as_u64() == target_depth
            })
        {
            return Value::Null;
        }
        let mut edits = Vec::new();
        for item in report["declarations"].as_array().into_iter().flatten().chain(report["references"].as_array().into_iter().flatten()) {
            if item["bindingId"].as_u64() != Some(binding_id) { continue; }
            let line = item["line"].as_u64().unwrap_or(1) as usize;
            let column = item["column"].as_u64().unwrap_or(1) as usize;
            let name = item["name"].as_str().unwrap_or("");
            let start = json!({ "line": line.saturating_sub(1), "character": utf16_column(source, line, column) });
            let end = json!({ "line": line.saturating_sub(1), "character": utf16_column(source, line, column) + name.encode_utf16().count() });
            edits.push(json!({ "range": { "start": start, "end": end }, "newText": new_name }));
        }
        (!edits.is_empty()).then(|| json!({ "changes": { uri: edits } })).unwrap_or(Value::Null)
    }
}

fn word_at(source: &str, position: &Value) -> Option<String> {
    let range = word_range_at(source, position)?;
    let line = range["start"]["line"].as_u64()? as usize;
    let start = range["start"]["character"].as_u64()? as usize;
    let end = range["end"]["character"].as_u64()? as usize;
    let text = source.lines().nth(line)?;
    let mut seen = 0usize;
    let selected: String = text.chars().filter(|character| { let width = character.len_utf16(); let include = seen >= start && seen < end; seen += width; include }).collect();
    (!selected.is_empty()).then_some(selected)
}

fn word_range_at(source: &str, position: &Value) -> Option<Value> {
    let line = position["line"].as_u64()? as usize;
    let character = position["character"].as_u64()? as usize;
    let text = source.lines().nth(line)?;
    let mut byte = 0usize;
    let mut utf16 = 0usize;
    for value in text.chars() {
        if utf16 >= character { break; }
        utf16 += value.len_utf16();
        byte += value.len_utf8();
    }
    let is_word = |value: char| value == '_' || value.is_alphanumeric() || ('\u{0980}'..='\u{09ff}').contains(&value);
    let mut start = byte.min(text.len());
    while start > 0 && is_word(text[..start].chars().next_back()?) {
        start = text[..start].char_indices().next_back()?.0;
    }
    let mut end = byte.min(text.len());
    while end < text.len() && is_word(text[end..].chars().next()?) {
        end += text[end..].chars().next()?.len_utf8();
    }
    (start < end).then(|| json!({
        "start": { "line": line, "character": text[..start].encode_utf16().count() },
        "end": { "line": line, "character": text[..end].encode_utf16().count() }
    }))
}

fn is_valid_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else { return false; };
    let valid = |character: char| character == '_' || character.is_alphanumeric() || ('\u{0980}'..='\u{09ff}').contains(&character);
    valid(first) && characters.all(valid) && !matches!(value, "let" | "ধরি" | "function" | "ফাংশন" | "if" | "যদি" | "for" | "প্রতি" | "while" | "যতক্ষণ")
}

fn document_symbols(source: &str) -> Vec<DocumentSymbol> {
    let report = match padma_lang::local_declarations_json(source) {
        Ok(Value::Array(items)) => items,
        _ => return Vec::new(),
    };
    report
        .into_iter()
        .filter_map(|item| {
            let name = item["name"].as_str()?.to_string();
            let kind = match item["kind"].as_str()? {
                "variable" => "variable",
                "loop-variable" => "loop variable",
                "function" => "function",
                _ => return None,
            };
            let one_based_line = item["line"].as_u64()? as usize;
            let one_based_column = item["column"].as_u64()? as usize;
            Some(DocumentSymbol {
                name,
                kind,
                line: one_based_line.saturating_sub(1),
                character: utf16_column(source, one_based_line, one_based_column),
                scope_depth: item["scopeDepth"].as_u64()? as usize,
            })
        })
        .collect()
}

fn scope_depth_at(source: &str, target_line: usize) -> usize {
    source.lines().take(target_line).fold(0usize, |depth, line| {
        depth.saturating_add(line.matches('{').count()).saturating_sub(line.matches('}').count())
    })
}

fn lsp_diagnostic(source: &str, diagnostic: &Value) -> Value {
    let range = &diagnostic["range"];
    let start_line = range["start"]["line"].as_u64().unwrap_or(1) as usize;
    let start_column = range["start"]["column"].as_u64().unwrap_or(1) as usize;
    let end_line = range["end"]["line"].as_u64().unwrap_or(start_line as u64) as usize;
    let end_column = range["end"]["column"].as_u64().unwrap_or(start_column as u64) as usize;
    json!({
        "range": {
            "start": { "line": start_line.saturating_sub(1), "character": utf16_column(source, start_line, start_column) },
            "end": { "line": end_line.saturating_sub(1), "character": utf16_column(source, end_line, end_column) }
        },
        "severity": 1,
        "code": diagnostic["code"],
        "source": "Padma",
        "message": diagnostic["message"]
    })
}

fn utf16_column(source: &str, one_based_line: usize, one_based_column: usize) -> usize {
    source
        .lines()
        .nth(one_based_line.saturating_sub(1))
        .unwrap_or("")
        .chars()
        .take(one_based_column.saturating_sub(1))
        .map(char::len_utf16)
        .sum()
}

fn end_position(source: &str) -> Value {
    let mut lines = source.split('\n');
    let mut line = 0usize;
    let mut last = "";
    if let Some(first) = lines.next() {
        last = first;
    }
    for next in lines {
        line += 1;
        last = next;
    }
    json!({ "line": line, "character": last.encode_utf16().count() })
}

fn completion_items() -> Value {
    let items = [
        ("let", "keyword"), ("ধরি", "keyword"), ("print", "keyword"), ("দেখাও", "keyword"),
        ("function", "keyword"), ("ফাংশন", "keyword"), ("return", "keyword"), ("ফেরত", "keyword"),
        ("if", "keyword"), ("যদি", "keyword"), ("else", "keyword"), ("নইলে", "keyword"),
        ("while", "keyword"), ("যতক্ষণ", "keyword"), ("for", "keyword"), ("প্রতি", "keyword"),
        ("input", "function"), ("range", "function"), ("পরিসর", "function"), ("text", "module"),
        ("math", "module"), ("json", "module"), ("file", "module"),
    ];
    Value::Array(items.into_iter().map(|(label, detail)| json!({ "label": label, "detail": detail })).collect())
}

fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }
    let Some(content_length) = content_length else {
        return Ok(None);
    };
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    Ok(serde_json::from_slice(&body).ok())
}

fn write_message(output: &mut impl Write, value: &Value) -> io::Result<()> {
    let body = value.to_string();
    write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    output.flush()
}

fn response(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = BufReader::new(stdin.lock());
    let mut output = stdout.lock();
    let mut server = Server::new();

    while let Some(message) = read_message(&mut input)? {
        let method = message["method"].as_str().unwrap_or("");
        let id = message.get("id").cloned();
        let params = &message["params"];
        match method {
            "initialize" => {
                if let Some(id) = id {
                    write_message(&mut output, &response(id, json!({
                        "capabilities": {
                            "textDocumentSync": 1,
                            "documentFormattingProvider": true,
                            "completionProvider": { "triggerCharacters": ["."] },
                            "hoverProvider": true,
                            "definitionProvider": true,
                            "renameProvider": { "prepareProvider": true }
                        },
                        "serverInfo": { "name": "padma-lsp", "version": "0.1.0" }
                    })))?;
                }
            }
            "initialized" => {}
            "shutdown" => {
                if let Some(id) = id {
                    write_message(&mut output, &response(id, Value::Null))?;
                }
            }
            "exit" => break,
            "textDocument/didOpen" | "textDocument/didChange" => {
                let document = if method == "textDocument/didOpen" {
                    &params["textDocument"]
                } else {
                    &params["textDocument"]
                };
                let uri = document["uri"].as_str().unwrap_or("");
                let text = if method == "textDocument/didOpen" {
                    document["text"].as_str().unwrap_or("")
                } else {
                    params["contentChanges"]
                        .as_array()
                        .and_then(|changes| changes.last())
                        .and_then(|change| change["text"].as_str())
                        .unwrap_or("")
                };
                server.documents.insert(uri.to_string(), text.to_string());
                server.publish_diagnostics(uri, &mut output)?;
            }
            "textDocument/didClose" => {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                server.documents.remove(uri);
                write_message(&mut output, &json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": { "uri": uri, "diagnostics": [] }
                }))?;
            }
            "textDocument/formatting" => {
                if let Some(id) = id {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    write_message(&mut output, &response(id, server.format_document(uri)))?;
                }
            }
            "textDocument/completion" => {
                if let Some(id) = id {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    write_message(&mut output, &response(id, server.completion(uri, &params["position"])))?;
                }
            }
            "textDocument/hover" => {
                if let Some(id) = id {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    write_message(&mut output, &response(id, server.hover(uri, &params["position"])))?;
                }
            }
            "textDocument/definition" => {
                if let Some(id) = id {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    write_message(&mut output, &response(id, server.definition(uri, &params["position"])))?;
                }
            }
            "textDocument/prepareRename" => {
                if let Some(id) = id {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    write_message(&mut output, &response(id, server.prepare_rename(uri, &params["position"])))?;
                }
            }
            "textDocument/rename" => {
                if let Some(id) = id {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let new_name = params["newName"].as_str().unwrap_or("");
                    write_message(&mut output, &response(id, server.rename(uri, &params["position"], new_name)))?;
                }
            }
            _ if id.is_some() => {
                write_message(&mut output, &json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "Method not found" }
                }))?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_bangla_positions_to_utf16_columns() {
        assert_eq!(utf16_column("ধরি = 1", 1, 4), 3);
    }

    #[test]
    fn produces_cli_compatible_lsp_error() {
        let diagnostic = json!({
            "code": "P1011",
            "message": "Cannot divide by zero.",
            "range": { "start": { "line": 1, "column": 7 }, "end": { "line": 1, "column": 8 } }
        });
        let output = lsp_diagnostic("print 1 / 0", &diagnostic);
        assert_eq!(output["code"], "P1011");
        assert_eq!(output["severity"], 1);
    }

    #[test]
    fn provides_bangla_and_english_static_completion_items() {
        let items = completion_items();
        assert!(items.as_array().unwrap().iter().any(|item| item["label"] == "ধরি"));
        assert!(items.as_array().unwrap().iter().any(|item| item["label"] == "function"));
    }

    #[test]
    fn extracts_bangla_words_for_hover() {
        let source = "ধরি নাম = \"রাফি\"\n";
        assert_eq!(word_at(source, &json!({ "line": 0, "character": 1 })), Some("ধরি".to_string()));
    }

    #[test]
    fn indexes_local_bangla_and_english_declarations_with_scopes() {
        let symbols = document_symbols("let outer = 1\nif true {\n  ধরি ভিতর = 2\n}\nfunction run() {\n}\n");
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0].name, "outer");
        assert_eq!(symbols[1].name, "ভিতর");
        assert_eq!(symbols[1].scope_depth, 1);
        assert_eq!(symbols[2].kind, "function");
    }

    #[test]
    fn resolves_the_nearest_visible_local_declaration() {
        let mut server = Server::new();
        let uri = "file:///demo.pd";
        server.documents.insert(uri.to_string(), "let name = 1\nif true {\n  ধরি নাম = 2\n  দেখাও নাম\n}\n".to_string());
        let location = server.definition(uri, &json!({ "line": 3, "character": 9 }));
        assert_eq!(location["range"]["start"]["line"], 2);
    }

    #[test]
    fn completes_visible_local_declarations() {
        let mut server = Server::new();
        let uri = "file:///demo.pd";
        server.documents.insert(uri.to_string(), "ধরি নাম = 1\nদেখাও নাম\n".to_string());
        let items = server.completion(uri, &json!({ "line": 1, "character": 0 }));
        assert!(items.as_array().unwrap().iter().any(|item| item["label"] == "নাম"));
    }

    #[test]
    fn renames_only_one_bound_local_symbol_with_bangla_shadowing() {
        let mut server = Server::new();
        let uri = "file:///rename.pd";
        server.documents.insert(uri.to_string(), "ধরি নাম = 1\nধরি অন্য = 3\nদেখাও নাম\nযদি সত্য {\n  ধরি নাম = 2\n  দেখাও নাম\n}\n".to_string());
        let edit = server.rename(uri, &json!({ "line": 2, "character": 7 }), "শিরোনাম");
        let changes = edit["changes"][uri].as_array().unwrap();
        assert_eq!(changes.len(), 2);
        assert!(changes.iter().all(|item| item["newText"] == "শিরোনাম"));
        assert!(server.prepare_rename(uri, &json!({ "line": 0, "character": 4 })).is_object());
        assert!(server.rename(uri, &json!({ "line": 2, "character": 7 }), "যদি").is_null());
        assert!(server.rename(uri, &json!({ "line": 2, "character": 7 }), "অন্য").is_null());
        assert!(server.prepare_rename(uri, &json!({ "line": 3, "character": 1 })).is_null());
    }
}
