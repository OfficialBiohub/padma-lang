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
}

fn word_at(source: &str, position: &Value) -> Option<String> {
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
    (start < end).then(|| text[start..end].to_string())
}

fn document_symbols(source: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    let mut depth = 0usize;
    for (line_number, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim_start();
        let indent = raw_line.len() - line.len();
        for (prefix, kind) in [("let ", "variable"), ("ধরি ", "variable"), ("function ", "function"), ("ফাংশন ", "function")] {
            if let Some(rest) = line.strip_prefix(prefix) {
                let name: String = rest.chars().take_while(|value| value == &'_' || value.is_alphanumeric() || ('\u{0980}'..='\u{09ff}').contains(value)).collect();
                if !name.is_empty() {
                    symbols.push(DocumentSymbol {
                        character: raw_line[..indent].encode_utf16().count() + prefix.encode_utf16().count(),
                        name,
                        kind,
                        line: line_number,
                        scope_depth: depth,
                    });
                }
                break;
            }
        }
        // Padma blocks are brace-delimited. This lexical counter intentionally does not
        // claim semantic scope for malformed syntax; parser diagnostics remain authoritative.
        depth = depth.saturating_add(line.matches('{').count());
        depth = depth.saturating_sub(line.matches('}').count());
    }
    symbols
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
                            "definitionProvider": true
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
                    write_message(&mut output, &response(id, completion_items()))?;
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
}
