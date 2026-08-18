use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};

use serde_json::{json, Value};

struct Server {
    documents: HashMap<String, String>,
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
                            "completionProvider": { "triggerCharacters": ["."] }
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
}
