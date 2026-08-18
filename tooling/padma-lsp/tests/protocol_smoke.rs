use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn write_message(writer: &mut impl Write, value: &Value) {
    let body = value.to_string();
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
    writer.flush().unwrap();
}

fn read_message(reader: &mut impl BufRead) -> Value {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = Some(value.trim().parse::<usize>().unwrap());
        }
    }
    let mut body = vec![0; content_length.expect("LSP response must have Content-Length")];
    reader.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[test]
fn advertises_and_executes_conservative_local_rename_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_padma-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let uri = "file:///sample.pd";

    write_message(&mut stdin, &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }));
    let initialize = read_message(&mut stdout);
    assert_eq!(initialize["id"], 1);
    assert_eq!(initialize["result"]["capabilities"]["renameProvider"]["prepareProvider"], true);

    let source = "ধরি নাম = 1\nদেখাও নাম\nযদি সত্য {\n  ধরি নাম = 2\n  দেখাও নাম\n}\n";
    write_message(&mut stdin, &json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": uri, "languageId": "padma", "version": 1, "text": source } }
    }));
    let diagnostics = read_message(&mut stdout);
    assert_eq!(diagnostics["method"], "textDocument/publishDiagnostics");

    write_message(&mut stdin, &json!({
        "jsonrpc": "2.0", "id": 2, "method": "textDocument/prepareRename",
        "params": { "textDocument": { "uri": uri }, "position": { "line": 0, "character": 4 } }
    }));
    let prepare = read_message(&mut stdout);
    assert_eq!(prepare["id"], 2);
    assert_eq!(prepare["result"]["start"]["character"], 4);

    write_message(&mut stdin, &json!({
        "jsonrpc": "2.0", "id": 3, "method": "textDocument/rename",
        "params": { "textDocument": { "uri": uri }, "position": { "line": 1, "character": 7 }, "newName": "শিরোনাম" }
    }));
    let rename = read_message(&mut stdout);
    assert_eq!(rename["id"], 3);
    assert_eq!(rename["result"]["changes"][uri].as_array().unwrap().len(), 2);

    write_message(&mut stdin, &json!({ "jsonrpc": "2.0", "id": 4, "method": "shutdown", "params": {} }));
    let shutdown = read_message(&mut stdout);
    assert_eq!(shutdown["id"], 4);
    drop(stdin);
    assert!(child.wait().unwrap().success());
}
