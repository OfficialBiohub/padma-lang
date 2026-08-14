// Padma v0.1.0 — a small, dependency-free Bangla-English language MVP.
//
// This executable intentionally implements a narrow but complete vertical slice:
// UTF-8 source, Bangla/English keyword aliases, expressions, variables, print,
// conditionals, string interpolation, and localized diagnostics.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Locale {
    Bangla,
    English,
}

impl Locale {
    fn from_source(source: &str) -> Self {
        if source.contains("padma:locale=en") {
            return Self::English;
        }
        if source.contains("padma:locale=bn") {
            return Self::Bangla;
        }

        let bangla = ["ধরি", "দেখাও", "যদি", "নইলে", "সত্য", "মিথ্যা"]
            .iter()
            .map(|word| source.matches(word).count())
            .sum::<usize>();
        let english = ["let", "print", "if", "else", "true", "false"]
            .iter()
            .map(|word| source.matches(word).count())
            .sum::<usize>();

        if english > bangla {
            Self::English
        } else {
            Self::Bangla
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Position {
    line: usize,
    column: usize,
}

impl Position {
    const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Debug)]
struct PadmaError {
    code: &'static str,
    message: String,
    hint: Option<String>,
    position: Position,
}

impl PadmaError {
    fn new(
        code: &'static str,
        message: impl Into<String>,
        hint: Option<String>,
        position: Position,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            hint,
            position,
        }
    }
}

fn error_for(locale: Locale, code: &'static str, position: Position, detail: &str) -> PadmaError {
    let (message, hint) = match (locale, code) {
        (Locale::Bangla, "P1001") => (
            format!("অচেনা চিহ্ন `{detail}`"),
            Some("শুধু অনুমোদিত Padma চিহ্ন ব্যবহার করুন।".into()),
        ),
        (Locale::English, "P1001") => (
            format!("Unexpected character `{detail}`"),
            Some("Use only supported Padma characters.".into()),
        ),
        (Locale::Bangla, "P1002") => (
            "string শেষ হওয়ার আগে file শেষ হয়ে গেছে".into(),
            Some("string-এর শেষে একটি closing quote (\") যোগ করুন।".into()),
        ),
        (Locale::English, "P1002") => (
            "Reached the end of the file before the string was closed".into(),
            Some("Add a closing double quote (\").".into()),
        ),
        (Locale::Bangla, "P1003") => (
            format!("এখানে `{detail}` প্রত্যাশিত ছিল"),
            Some("আগের statement ও বন্ধনীগুলো পরীক্ষা করুন।".into()),
        ),
        (Locale::English, "P1003") => (
            format!("Expected `{detail}` here"),
            Some("Check the preceding statement and delimiters.".into()),
        ),
        (Locale::Bangla, "P1004") => (
            format!("এই statement শুরু করতে `{detail}` ব্যবহার করা যাবে না"),
            Some("`ধরি`, `দেখাও`, অথবা `যদি` দিয়ে নতুন statement শুরু করুন।".into()),
        ),
        (Locale::English, "P1004") => (
            format!("`{detail}` cannot start a statement"),
            Some("Start a statement with `let`, `print`, or `if`.".into()),
        ),
        (Locale::Bangla, "P1007") => (
            format!("`{detail}` নামে কোনো variable পাওয়া যায়নি"),
            Some(format!("আগে এটি ঘোষণা করুন: `ধরি {detail} = ...`")),
        ),
        (Locale::English, "P1007") => (
            format!("Cannot find variable `{detail}`"),
            Some(format!("Declare it first: `let {detail} = ...`")),
        ),
        (Locale::Bangla, "P1010") => (
            format!("`{detail}` অপারেশনের জন্য মানগুলোর ধরন মিলছে না"),
            Some("সংখ্যার সঙ্গে সংখ্যা বা লেখার সঙ্গে লেখা ব্যবহার করুন।".into()),
        ),
        (Locale::English, "P1010") => (
            format!("Values do not have compatible types for `{detail}`"),
            Some("Use numbers with numbers or text with text.".into()),
        ),
        (Locale::Bangla, "P1011") => (
            "শূন্য দিয়ে ভাগ করা যাবে না".into(),
            Some("ভাজকটি শূন্য কি না আগে `যদি` দিয়ে পরীক্ষা করুন।".into()),
        ),
        (Locale::English, "P1011") => (
            "Cannot divide by zero".into(),
            Some("Use `if` to check that the divisor is not zero first.".into()),
        ),
        (Locale::Bangla, "P1012") => (
            "loop নির্ধারিত iteration সীমা অতিক্রম করেছে".into(),
            Some("condition ও loop-এর variable update পরীক্ষা করুন।".into()),
        ),
        (Locale::English, "P1012") => (
            "The loop exceeded the configured iteration limit".into(),
            Some("Check the condition and update the loop variable.".into()),
        ),
        (Locale::Bangla, "P1014") => (
            format!("নিরাপত্তার কারণে output path অনুমোদিত নয়: `{detail}`"),
            Some("বর্তমান folder-এর ভেতরের relative path ব্যবহার করুন; `..` বা absolute path ব্যবহার করবেন না।".into()),
        ),
        (Locale::English, "P1014") => (
            format!("Output path is not allowed for safety: `{detail}`"),
            Some("Use a relative path inside the current folder; do not use `..` or an absolute path.".into()),
        ),
        (Locale::Bangla, "P1015") => (format!("file লেখা যায়নি: `{detail}`"), Some("folder ও permission পরীক্ষা করুন।".into())),
        (Locale::English, "P1015") => (format!("Could not write file: `{detail}`"), Some("Check the folder and permissions.".into())),
        (Locale::Bangla, "P1016") => (format!("URL অনুমোদিত নয়: `{detail}`"), Some("http:// অথবা https:// URL ব্যবহার করুন।".into())),
        (Locale::English, "P1016") => (format!("URL is not allowed: `{detail}`"), Some("Use an http:// or https:// URL.".into())),
        (Locale::Bangla, "P1017") => (format!("এই process অনুমোদিত নয়: `{detail}`"), Some("শুধু অনুমোদিত downloader/tool ব্যবহার করুন।".into())),
        (Locale::English, "P1017") => (format!("This process is not allowed: `{detail}`"), Some("Use only an approved downloader/tool.".into())),
        (Locale::Bangla, "P1018") => (format!("process চালু করা যায়নি: `{detail}`"), Some("Termux-এ toolটি install আছে কি না পরীক্ষা করুন।".into())),
        (Locale::English, "P1018") => (format!("Could not start process: `{detail}`"), Some("Check that the tool is installed in Termux.".into())),
        (Locale::Bangla, "P1019") => (format!("process ব্যর্থ হয়েছে: `{detail}`"), Some("tool-এর output ও URL পরীক্ষা করুন।".into())),
        (Locale::English, "P1019") => (format!("Process failed: `{detail}`"), Some("Check the tool output and URL.".into())),
        (Locale::Bangla, "P1020") => (format!("map-এর key অবশ্যই text হতে হবে: `{detail}`"), Some("উদাহরণ: `map.get(\"নাম\")` ব্যবহার করুন।".into())),
        (Locale::English, "P1020") => (format!("Map keys must be text: `{detail}`"), Some("Use a string key, for example `map.get(\"name\")`.".into())),
        (Locale::Bangla, "P1021") => (format!("map-এ `{detail}` key পাওয়া যায়নি"), Some("আগে `map.set` দিয়ে key-টি যোগ করুন।".into())),
        (Locale::English, "P1021") => (format!("Map key `{detail}` was not found"), Some("Add the key first with `map.set`.".into())),
        (Locale::Bangla, "P1013") => ("input পড়া যায়নি".into(), Some("আবার চেষ্টা করুন।".into())),
        (Locale::English, "P1013") => ("Could not read input".into(), Some("Try again.".into())),
        _ => (format!("Internal Padma error: {detail}"), None),
    };
    PadmaError::new(code, message, hint, position)
}

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Let,
    Print,
    If,
    Else,
    While,
    Function,
    Return,
    True,
    False,
    Identifier(String),
    Number(f64),
    String(String),
    Equal,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Colon,
    LeftBracket,
    RightBracket,
    Newline,
    Eof,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    position: Position,
}

struct Lexer {
    chars: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    locale: Locale,
}

impl Lexer {
    fn new(source: &str, locale: Locale) -> Self {
        Self {
            chars: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            locale,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, PadmaError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            let position = self.position();
            let character = self.advance();
            match character {
                ' ' | '\t' | '\r' => {}
                '\n' => tokens.push(self.token(TokenKind::Newline, position)),
                '#' => self.consume_comment(),
                '"' => tokens.push(self.string(position)?),
                '=' => {
                    if self.matches('=') {
                        tokens.push(self.token(TokenKind::EqualEqual, position));
                    } else {
                        tokens.push(self.token(TokenKind::Equal, position));
                    }
                }
                '!' => {
                    if self.matches('=') {
                        tokens.push(self.token(TokenKind::BangEqual, position));
                    } else {
                        return Err(error_for(self.locale, "P1001", position, "!"));
                    }
                }
                '>' => {
                    if self.matches('=') {
                        tokens.push(self.token(TokenKind::GreaterEqual, position));
                    } else {
                        tokens.push(self.token(TokenKind::Greater, position));
                    }
                }
                '<' => {
                    if self.matches('=') {
                        tokens.push(self.token(TokenKind::LessEqual, position));
                    } else {
                        tokens.push(self.token(TokenKind::Less, position));
                    }
                }
                '+' => tokens.push(self.token(TokenKind::Plus, position)),
                '-' => tokens.push(self.token(TokenKind::Minus, position)),
                '*' => tokens.push(self.token(TokenKind::Star, position)),
                '/' => tokens.push(self.token(TokenKind::Slash, position)),
                '(' => tokens.push(self.token(TokenKind::LeftParen, position)),
                ')' => tokens.push(self.token(TokenKind::RightParen, position)),
                '{' => tokens.push(self.token(TokenKind::LeftBrace, position)),
                '}' => tokens.push(self.token(TokenKind::RightBrace, position)),
                ',' => tokens.push(self.token(TokenKind::Comma, position)),
                '.' => tokens.push(self.token(TokenKind::Dot, position)),
                ':' => tokens.push(self.token(TokenKind::Colon, position)),
                '[' => tokens.push(self.token(TokenKind::LeftBracket, position)),
                ']' => tokens.push(self.token(TokenKind::RightBracket, position)),
                ch if is_digit(ch) => tokens.push(self.number(position)?),
                ch if is_identifier_start(ch) => tokens.push(self.identifier(position)),
                ch => return Err(error_for(self.locale, "P1001", position, &ch.to_string())),
            }
        }
        tokens.push(self.token(TokenKind::Eof, self.position()));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    fn token(&self, kind: TokenKind, position: Position) -> Token {
        Token { kind, position }
    }

    fn advance(&mut self) -> char {
        let value = self.chars[self.current];
        self.current += 1;
        if value == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        value
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.current).copied()
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume_comment(&mut self) {
        while let Some(character) = self.peek() {
            if character == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn string(&mut self, position: Position) -> Result<Token, PadmaError> {
        let mut value = String::new();
        while let Some(character) = self.peek() {
            if character == '"' {
                self.advance();
                return Ok(self.token(TokenKind::String(value), position));
            }
            if character == '\\' {
                self.advance();
                let escaped = self
                    .peek()
                    .ok_or_else(|| error_for(self.locale, "P1002", position, "string"))?;
                self.advance();
                value.push(match escaped {
                    'n' => '\n',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => other,
                });
            } else {
                value.push(self.advance());
            }
        }
        Err(error_for(self.locale, "P1002", position, "string"))
    }

    fn number(&mut self, position: Position) -> Result<Token, PadmaError> {
        let mut raw = String::new();
        raw.push(normalize_digit(self.chars[self.current - 1]));
        while let Some(character) = self.peek() {
            if is_digit(character) {
                raw.push(normalize_digit(self.advance()));
            } else if character == '.' {
                raw.push(self.advance());
            } else {
                break;
            }
        }
        match raw.parse::<f64>() {
            Ok(number) => Ok(self.token(TokenKind::Number(number), position)),
            Err(_) => Err(error_for(self.locale, "P1001", position, &raw)),
        }
    }

    fn identifier(&mut self, position: Position) -> Token {
        let mut word = String::new();
        word.push(self.chars[self.current - 1]);
        while let Some(character) = self.peek() {
            if is_identifier_continue(character) {
                word.push(self.advance());
            } else {
                break;
            }
        }
        let kind = match word.as_str() {
            "ধরি" | "let" => TokenKind::Let,
            "দেখাও" | "print" => TokenKind::Print,
            "যদি" | "if" => TokenKind::If,
            "নইলে" | "else" => TokenKind::Else,
            "যতক্ষণ" | "while" => TokenKind::While,
            "ফাংশন" | "function" | "fn" => TokenKind::Function,
            "ফেরত" | "return" => TokenKind::Return,
            "সত্য" | "true" => TokenKind::True,
            "মিথ্যা" | "false" => TokenKind::False,
            _ => TokenKind::Identifier(word),
        };
        self.token(kind, position)
    }
}

fn is_digit(character: char) -> bool {
    character.is_ascii_digit() || ('০'..='৯').contains(&character)
}

fn normalize_digit(character: char) -> char {
    match character {
        '০' => '0',
        '১' => '1',
        '২' => '2',
        '৩' => '3',
        '৪' => '4',
        '৫' => '5',
        '৬' => '6',
        '৭' => '7',
        '৮' => '8',
        '৯' => '9',
        other => other,
    }
}

fn is_bangla(character: char) -> bool {
    ('\u{0980}'..='\u{09ff}').contains(&character)
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic() || is_bangla(character)
}

fn is_identifier_continue(character: char) -> bool {
    is_identifier_start(character) || character.is_ascii_digit() || is_digit(character)
}

fn resolve_output_path(path: &str) -> Result<PathBuf, ()> {
    if path.is_empty() || path.split('/').any(|part| part == "..") {
        return Err(());
    }
    if path == "@downloads" || path.starts_with("@downloads/") {
        let home = env::var_os("HOME").ok_or(())?;
        let relative = path
            .strip_prefix("@downloads")
            .unwrap_or("")
            .trim_start_matches('/');
        return Ok(PathBuf::from(home).join("storage/downloads").join(relative));
    }
    let candidate = std::path::Path::new(path);
    if candidate.is_absolute() {
        return Err(());
    }
    Ok(PathBuf::from(path))
}

fn expect_string<'a>(
    value: &'a Value,
    locale: Locale,
    position: Position,
    label: &str,
) -> Result<&'a str, PadmaError> {
    match value {
        Value::String(value) => Ok(value),
        _ => Err(error_for(locale, "P1010", position, label)),
    }
}

fn expect_map_key<'a>(
    value: &'a Value,
    locale: Locale,
    position: Position,
) -> Result<&'a str, PadmaError> {
    match value {
        Value::String(value) => Ok(value),
        other => Err(error_for(locale, "P1020", position, &other.to_string())),
    }
}
#[derive(Clone, Debug)]
enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Print {
        value: Expr,
    },
    Expression {
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
        position: Position,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        position: Position,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
    },
}

#[derive(Clone, Debug)]
enum Expr {
    Literal(Value, Position),
    Variable(String, Position),
    Unary {
        operator: TokenKind,
        right: Box<Expr>,
        position: Position,
    },
    Binary {
        left: Box<Expr>,
        operator: TokenKind,
        right: Box<Expr>,
        position: Position,
    },
    Call {
        name: String,
        arguments: Vec<Expr>,
        position: Position,
    },
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    locale: Locale,
}

impl Parser {
    fn new(tokens: Vec<Token>, locale: Locale) -> Self {
        Self {
            tokens,
            current: 0,
            locale,
        }
    }

    fn parse(mut self) -> Result<Vec<Stmt>, PadmaError> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            statements.push(self.statement()?);
            self.skip_newlines();
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, PadmaError> {
        if self.matches(|kind| matches!(kind, TokenKind::Let)) {
            return self.let_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::Print)) {
            return self.print_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::If)) {
            return self.if_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::While)) {
            return self.while_statement(self.previous().position);
        }
        if self.matches(|kind| matches!(kind, TokenKind::Function)) {
            return self.function_statement();
        }
        if self.matches(|kind| matches!(kind, TokenKind::Return)) {
            return self.return_statement(self.previous().position);
        }
        if self.check(|kind| matches!(kind, TokenKind::Identifier(_)))
            && self
                .tokens
                .get(self.current + 1)
                .is_some_and(|token| matches!(token.kind, TokenKind::Equal))
        {
            return self.assign_statement();
        }
        let value = self.expression()?;
        self.consume_statement_end()?;
        Ok(Stmt::Expression { value })
    }

    fn let_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let name = match self.advance().clone() {
            Token {
                kind: TokenKind::Identifier(name),
                ..
            } => name,
            token => {
                return Err(error_for(
                    self.locale,
                    "P1003",
                    token.position,
                    "variable name",
                ))
            }
        };
        self.consume(|kind| matches!(kind, TokenKind::Equal), "=")?;
        let value = self.expression()?;
        self.consume_statement_end()?;
        let _ = position;
        Ok(Stmt::Let { name, value })
    }

    fn print_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let value = self.expression()?;
        self.consume_statement_end()?;
        let _ = position;
        Ok(Stmt::Print { value })
    }

    fn assign_statement(&mut self) -> Result<Stmt, PadmaError> {
        let token = self.advance().clone();
        let name = match token.kind {
            TokenKind::Identifier(name) => name,
            _ => unreachable!("assignment lookahead guarantees an identifier"),
        };
        self.consume(|kind| matches!(kind, TokenKind::Equal), "=")?;
        let value = self.expression()?;
        self.consume_statement_end()?;
        Ok(Stmt::Assign {
            name,
            value,
            position: token.position,
        })
    }

    fn if_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let condition = self.expression()?;
        self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let then_branch = self.block()?;
        self.skip_newlines();
        let else_branch = if self.matches(|kind| matches!(kind, TokenKind::Else)) {
            self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
            self.block()?
        } else {
            Vec::new()
        };
        let _ = position;
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let condition = self.expression()?;
        self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let body = self.block()?;
        Ok(Stmt::While {
            condition,
            body,
            position,
        })
    }

    fn function_statement(&mut self) -> Result<Stmt, PadmaError> {
        let name = match self.advance().clone().kind {
            TokenKind::Identifier(name) => name,
            _token => {
                return Err(error_for(
                    self.locale,
                    "P1003",
                    self.previous().position,
                    "function name",
                ))
            }
        };
        self.consume(|kind| matches!(kind, TokenKind::LeftParen), "(")?;
        let mut params = Vec::new();
        if !self.check(|kind| matches!(kind, TokenKind::RightParen)) {
            loop {
                match self.advance().clone().kind {
                    TokenKind::Identifier(name) => params.push(name),
                    _ => {
                        return Err(error_for(
                            self.locale,
                            "P1003",
                            self.previous().position,
                            "parameter name",
                        ))
                    }
                }
                if !self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                    break;
                }
            }
        }
        self.consume(|kind| matches!(kind, TokenKind::RightParen), ")")?;
        self.consume(|kind| matches!(kind, TokenKind::LeftBrace), "{")?;
        let body = self.block()?;
        Ok(Stmt::Function { name, params, body })
    }

    fn return_statement(&mut self, position: Position) -> Result<Stmt, PadmaError> {
        let value = if self.check(|kind| {
            matches!(
                kind,
                TokenKind::Newline | TokenKind::RightBrace | TokenKind::Eof
            )
        }) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume_statement_end()?;
        let _ = position;
        Ok(Stmt::Return { value })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, PadmaError> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.check(|kind| matches!(kind, TokenKind::RightBrace)) && !self.is_at_end() {
            statements.push(self.statement()?);
            self.skip_newlines();
        }
        self.consume(|kind| matches!(kind, TokenKind::RightBrace), "}")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, PadmaError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.comparison()?;
        while self.matches(|kind| matches!(kind, TokenKind::EqualEqual | TokenKind::BangEqual)) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            };
        }
        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.term()?;
        while self.matches(|kind| {
            matches!(
                kind,
                TokenKind::Greater
                    | TokenKind::GreaterEqual
                    | TokenKind::Less
                    | TokenKind::LessEqual
            )
        }) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            };
        }
        Ok(expression)
    }

    fn term(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.factor()?;
        while self.matches(|kind| matches!(kind, TokenKind::Plus | TokenKind::Minus)) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            };
        }
        Ok(expression)
    }

    fn factor(&mut self) -> Result<Expr, PadmaError> {
        let mut expression = self.unary()?;
        while self.matches(|kind| matches!(kind, TokenKind::Star | TokenKind::Slash)) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expression = Expr::Binary {
                left: Box::new(expression),
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            };
        }
        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expr, PadmaError> {
        if self.matches(|kind| matches!(kind, TokenKind::Minus)) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator: operator.kind,
                right: Box::new(right),
                position: operator.position,
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, PadmaError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number(value) => Ok(Expr::Literal(Value::Number(value), token.position)),
            TokenKind::String(value) => Ok(Expr::Literal(Value::String(value), token.position)),
            TokenKind::True => Ok(Expr::Literal(Value::Boolean(true), token.position)),
            TokenKind::False => Ok(Expr::Literal(Value::Boolean(false), token.position)),
            TokenKind::Identifier(name) => {
                let name = if self.matches(|kind| matches!(kind, TokenKind::Dot)) {
                    match self.advance().clone().kind {
                        TokenKind::Identifier(member) => format!("{name}.{member}"),
                        _ => {
                            return Err(error_for(
                                self.locale,
                                "P1003",
                                token.position,
                                "member name",
                            ))
                        }
                    }
                } else {
                    name
                };
                if self.matches(|kind| matches!(kind, TokenKind::LeftParen)) {
                    let mut arguments = Vec::new();
                    if !self.check(|kind| matches!(kind, TokenKind::RightParen)) {
                        loop {
                            arguments.push(self.expression()?);
                            if !self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                                break;
                            }
                        }
                    }
                    self.consume(|kind| matches!(kind, TokenKind::RightParen), ")")?;
                    Ok(Expr::Call {
                        name,
                        arguments,
                        position: token.position,
                    })
                } else {
                    Ok(Expr::Variable(name, token.position))
                }
            }
            TokenKind::LeftBracket => {
                let mut values = Vec::new();
                if !self.check(|kind| matches!(kind, TokenKind::RightBracket)) {
                    loop {
                        values.push(self.expression()?);
                        if !self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                            break;
                        }
                    }
                }
                self.consume(|kind| matches!(kind, TokenKind::RightBracket), "]")?;
                Ok(Expr::List(values))
            }
            TokenKind::LeftBrace => {
                let mut entries = Vec::new();
                if !self.check(|kind| matches!(kind, TokenKind::RightBrace)) {
                    loop {
                        let key = self.expression()?;
                        self.consume(|kind| matches!(kind, TokenKind::Colon), ":")?;
                        let value = self.expression()?;
                        entries.push((key, value));
                        if !self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                            break;
                        }
                    }
                }
                self.consume(|kind| matches!(kind, TokenKind::RightBrace), "}")?;
                Ok(Expr::Map(entries))
            }
            TokenKind::LeftParen => {
                let expression = self.expression()?;
                self.consume(|kind| matches!(kind, TokenKind::RightParen), ")")?;
                Ok(expression)
            }
            _ => Err(error_for(
                self.locale,
                "P1003",
                token.position,
                "expression",
            )),
        }
    }

    fn consume_statement_end(&mut self) -> Result<(), PadmaError> {
        if self.check(|kind| {
            matches!(
                kind,
                TokenKind::Newline | TokenKind::RightBrace | TokenKind::Eof
            )
        }) {
            return Ok(());
        }
        let token = self.peek().clone();
        Err(error_for(self.locale, "P1003", token.position, "new line"))
    }

    fn consume<F>(&mut self, predicate: F, expected: &str) -> Result<Token, PadmaError>
    where
        F: Fn(&TokenKind) -> bool,
    {
        if self.check(predicate) {
            Ok(self.advance().clone())
        } else {
            let token = self.peek().clone();
            Err(error_for(self.locale, "P1003", token.position, expected))
        }
    }

    fn skip_newlines(&mut self) {
        while self.matches(|kind| matches!(kind, TokenKind::Newline)) {}
    }

    fn matches<F>(&mut self, predicate: F) -> bool
    where
        F: Fn(&TokenKind) -> bool,
    {
        if self.check(predicate) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check<F>(&self, predicate: F) -> bool
    where
        F: Fn(&TokenKind) -> bool,
    {
        !self.is_at_end() && predicate(&self.peek().kind)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Self::Boolean(value) => *value,
            Self::Number(value) => *value != 0.0,
            Self::String(value) => !value.is_empty(),
            Self::List(value) => !value.is_empty(),
            Self::Map(value) => !value.is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) if value.fract() == 0.0 => write!(formatter, "{value:.0}"),
            Self::Number(value) => write!(formatter, "{value}"),
            Self::String(value) => write!(formatter, "{value}"),
            Self::Boolean(true) => write!(formatter, "true"),
            Self::Boolean(false) => write!(formatter, "false"),
            Self::List(values) => {
                write!(formatter, "[")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "{value}")?;
                }
                write!(formatter, "]")
            }
            Self::Map(values) => {
                write!(formatter, "{{")?;
                for (index, (key, value)) in values.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, ", ")?;
                    }
                    write!(formatter, "\"{key}\": {value}")?;
                }
                write!(formatter, "}}")
            }
        }
    }
}

struct Interpreter {
    environment: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    return_value: Option<Value>,
    output: Vec<String>,
    locale: Locale,
}

impl Interpreter {
    fn new(locale: Locale) -> Self {
        Self {
            environment: HashMap::new(),
            functions: HashMap::new(),
            return_value: None,
            output: Vec::new(),
            locale,
        }
    }

    fn run(&mut self, program: &[Stmt]) -> Result<(), PadmaError> {
        for statement in program {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), PadmaError> {
        match statement {
            Stmt::Let { name, value, .. } => {
                let value = self.evaluate(value)?;
                self.environment.insert(name.clone(), value);
            }
            Stmt::Print { value, .. } => {
                let value = self.evaluate(value)?;
                self.output.push(value.to_string());
            }
            Stmt::Expression { value } => {
                self.evaluate(value)?;
            }
            Stmt::Assign {
                name,
                value,
                position,
            } => {
                if !self.environment.contains_key(name) {
                    return Err(error_for(self.locale, "P1007", *position, name));
                }
                let value = self.evaluate(value)?;
                self.environment.insert(name.clone(), value);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let branch = if self.evaluate(condition)?.truthy() {
                    then_branch
                } else {
                    else_branch
                };
                self.run(branch)?;
            }
            Stmt::While {
                condition,
                body,
                position,
            } => {
                let mut iterations = 0usize;
                while self.evaluate(condition)?.truthy() {
                    iterations += 1;
                    if iterations > 1_000_000 {
                        return Err(error_for(
                            self.locale,
                            "P1012",
                            *position,
                            "loop iteration limit",
                        ));
                    }
                    self.run(body)?;
                }
            }
            Stmt::Function { name, params, body } => {
                self.functions
                    .insert(name.clone(), (params.clone(), body.clone()));
            }
            Stmt::Return { value, .. } => {
                self.return_value = Some(match value {
                    Some(expression) => self.evaluate(expression)?,
                    None => Value::String(String::new()),
                });
            }
        }
        Ok(())
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Value, PadmaError> {
        match expression {
            Expr::Literal(Value::String(value), position) => {
                Ok(Value::String(self.interpolate(value, *position)?))
            }
            Expr::Literal(value, _) => Ok(value.clone()),
            Expr::Variable(name, position) => self
                .environment
                .get(name)
                .cloned()
                .ok_or_else(|| error_for(self.locale, "P1007", *position, name)),
            Expr::Unary {
                operator,
                right,
                position,
            } => match (operator, self.evaluate(right)?) {
                (TokenKind::Minus, Value::Number(value)) => Ok(Value::Number(-value)),
                (TokenKind::Minus, _) => Err(error_for(self.locale, "P1010", *position, "-")),
                _ => unreachable!("parser only creates supported unary expressions"),
            },
            Expr::Binary {
                left,
                operator,
                right,
                position,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                self.binary(left, operator, right, *position)
            }
            Expr::Call {
                name,
                arguments,
                position,
            } => {
                if name == "file.write" {
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    if values.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let path = match &values[0] {
                        Value::String(value) => value,
                        _ => return Err(error_for(self.locale, "P1010", *position, "path")),
                    };
                    let content = match &values[1] {
                        Value::String(value) => value,
                        _ => return Err(error_for(self.locale, "P1010", *position, "content")),
                    };
                    let resolved_path = resolve_output_path(path)
                        .map_err(|_| error_for(self.locale, "P1014", *position, path))?;
                    fs::write(&resolved_path, content)
                        .map_err(|_| error_for(self.locale, "P1015", *position, path))?;
                    return Ok(Value::Boolean(true));
                }
                if name == "http.get" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let url = self.evaluate(&arguments[0])?;
                    let url = expect_string(&url, self.locale, *position, "url")?;
                    if !(url.starts_with("https://") || url.starts_with("http://")) {
                        return Err(error_for(self.locale, "P1016", *position, url));
                    }
                    let result = process::Command::new("curl")
                        .args([
                            "--fail",
                            "--silent",
                            "--show-error",
                            "--location",
                            "--max-time",
                            "30",
                            "--",
                            url,
                        ])
                        .output()
                        .map_err(|_| error_for(self.locale, "P1018", *position, "curl"))?;
                    if !result.status.success() {
                        return Err(error_for(self.locale, "P1019", *position, "curl"));
                    }
                    return Ok(Value::String(
                        String::from_utf8_lossy(&result.stdout).to_string(),
                    ));
                }
                if name == "process.run" || name == "media.download" {
                    let values = arguments
                        .iter()
                        .map(|argument| self.evaluate(argument))
                        .collect::<Result<Vec<_>, _>>()?;
                    let (program, args) = if name == "media.download" {
                        if values.len() != 1 && values.len() != 2 {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        let url = expect_string(&values[0], self.locale, *position, "url")?;
                        let output = if values.len() == 2 {
                            expect_string(&values[1], self.locale, *position, "output")?.to_string()
                        } else {
                            "@downloads/%(title)s.%(ext)s".to_string()
                        };
                        if !(url.starts_with("https://") || url.starts_with("http://")) {
                            return Err(error_for(self.locale, "P1016", *position, url));
                        }
                        let resolved_output = resolve_output_path(&output)
                            .map_err(|_| error_for(self.locale, "P1014", *position, &output))?;
                        (
                            "yt-dlp".to_string(),
                            vec![
                                "-o".to_string(),
                                resolved_output.to_string_lossy().to_string(),
                                url.to_string(),
                            ],
                        )
                    } else {
                        if values.is_empty() {
                            return Err(error_for(self.locale, "P1009", *position, name));
                        }
                        let program = expect_string(&values[0], self.locale, *position, "program")?;
                        let args = values[1..]
                            .iter()
                            .map(|value| {
                                expect_string(value, self.locale, *position, "argument")
                                    .map(str::to_string)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        (program.to_string(), args)
                    };
                    let allowed = ["yt-dlp", "curl", "ffmpeg", "python", "python3"];
                    if !allowed.contains(&program.as_str()) {
                        return Err(error_for(self.locale, "P1017", *position, &program));
                    }
                    let result = process::Command::new(&program)
                        .args(&args)
                        .output()
                        .map_err(|_| error_for(self.locale, "P1018", *position, &program))?;
                    if !result.status.success() {
                        return Err(error_for(self.locale, "P1019", *position, &program));
                    }
                    return Ok(Value::String(
                        String::from_utf8_lossy(&result.stdout).trim().to_string(),
                    ));
                }
                if name == "input" {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let prompt = self.evaluate(&arguments[0])?;
                    print!("{prompt}");
                    io::stdout()
                        .flush()
                        .map_err(|_| error_for(self.locale, "P1013", *position, "input"))?;
                    let mut line = String::new();
                    io::stdin()
                        .read_line(&mut line)
                        .map_err(|_| error_for(self.locale, "P1013", *position, "input"))?;
                    return Ok(Value::String(
                        line.trim_end_matches(['\r', '\n']).to_string(),
                    ));
                }
                if let Some(map_name) = name.strip_suffix(".get") {
                    if arguments.len() != 1 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let key = self.evaluate(&arguments[0])?;
                    let key = expect_map_key(&key, self.locale, *position)?;
                    let map = self
                        .environment
                        .get(map_name)
                        .ok_or_else(|| error_for(self.locale, "P1007", *position, map_name))?;
                    let Value::Map(values) = map else {
                        return Err(error_for(self.locale, "P1010", *position, "map.get"));
                    };
                    return values
                        .get(key)
                        .cloned()
                        .ok_or_else(|| error_for(self.locale, "P1021", *position, key));
                }
                if let Some(map_name) = name.strip_suffix(".set") {
                    if arguments.len() != 2 {
                        return Err(error_for(self.locale, "P1009", *position, name));
                    }
                    let key = self.evaluate(&arguments[0])?;
                    let key = expect_map_key(&key, self.locale, *position)?.to_owned();
                    let value = self.evaluate(&arguments[1])?;
                    let map = self
                        .environment
                        .get_mut(map_name)
                        .ok_or_else(|| error_for(self.locale, "P1007", *position, map_name))?;
                    let Value::Map(values) = map else {
                        return Err(error_for(self.locale, "P1010", *position, "map.set"));
                    };
                    values.insert(key, value);
                    return Ok(Value::Boolean(true));
                }
                let (parameters, body) = self
                    .functions
                    .get(name)
                    .cloned()
                    .ok_or_else(|| error_for(self.locale, "P1008", *position, name))?;
                if parameters.len() != arguments.len() {
                    return Err(error_for(self.locale, "P1009", *position, name));
                }
                let values = arguments
                    .iter()
                    .map(|argument| self.evaluate(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                let previous_environment = std::mem::replace(&mut self.environment, HashMap::new());
                let previous_return = self.return_value.take();
                for (parameter, value) in parameters.iter().zip(values) {
                    self.environment.insert(parameter.clone(), value);
                }
                self.run(&body)?;
                let result = self
                    .return_value
                    .take()
                    .unwrap_or(Value::String(String::new()));
                self.environment = previous_environment;
                self.return_value = previous_return;
                Ok(result)
            }
            Expr::List(expressions) => expressions
                .iter()
                .map(|expression| self.evaluate(expression))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::List),
            Expr::Map(entries) => {
                let mut values = BTreeMap::new();
                for (key, value) in entries {
                    let key = self.evaluate(key)?;
                    let key = expect_string(&key, self.locale, Position::new(1, 1), "map key")?;
                    values.insert(key.to_owned(), self.evaluate(value)?);
                }
                Ok(Value::Map(values))
            }
        }
    }

    fn binary(
        &self,
        left: Value,
        operator: &TokenKind,
        right: Value,
        position: Position,
    ) -> Result<Value, PadmaError> {
        use TokenKind::*;
        match operator {
            Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => Ok(Value::String(left + &right)),
                _ => Err(error_for(self.locale, "P1010", position, "+")),
            },
            Minus => numeric(self.locale, position, "-", left, right, |a, b| a - b),
            Star => numeric(self.locale, position, "*", left, right, |a, b| a * b),
            Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) if right == 0.0 => {
                    let _ = left;
                    Err(error_for(self.locale, "P1011", position, "/"))
                }
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Err(error_for(self.locale, "P1010", position, "/")),
            },
            EqualEqual => Ok(Value::Boolean(left == right)),
            BangEqual => Ok(Value::Boolean(left != right)),
            Greater => compare(self.locale, position, ">", left, right, |a, b| a > b),
            GreaterEqual => compare(self.locale, position, ">=", left, right, |a, b| a >= b),
            Less => compare(self.locale, position, "<", left, right, |a, b| a < b),
            LessEqual => compare(self.locale, position, "<=", left, right, |a, b| a <= b),
            _ => unreachable!("parser only creates supported binary expressions"),
        }
    }

    fn interpolate(&self, text: &str, position: Position) -> Result<String, PadmaError> {
        let mut result = String::new();
        let mut characters = text.chars().peekable();
        while let Some(character) = characters.next() {
            if character != '{' {
                result.push(character);
                continue;
            }
            let mut variable = String::new();
            let mut closed = false;
            for candidate in characters.by_ref() {
                if candidate == '}' {
                    closed = true;
                    break;
                }
                variable.push(candidate);
            }
            if !closed || variable.trim().is_empty() {
                result.push('{');
                result.push_str(&variable);
                if closed {
                    result.push('}');
                }
                continue;
            }
            let name = variable.trim();
            let value = self
                .environment
                .get(name)
                .ok_or_else(|| error_for(self.locale, "P1007", position, name))?;
            result.push_str(&value.to_string());
        }
        Ok(result)
    }
}

fn numeric<F>(
    locale: Locale,
    position: Position,
    operator: &str,
    left: Value,
    right: Value,
    operation: F,
) -> Result<Value, PadmaError>
where
    F: FnOnce(f64, f64) -> f64,
{
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => Ok(Value::Number(operation(left, right))),
        _ => Err(error_for(locale, "P1010", position, operator)),
    }
}

fn compare<F>(
    locale: Locale,
    position: Position,
    operator: &str,
    left: Value,
    right: Value,
    comparison: F,
) -> Result<Value, PadmaError>
where
    F: FnOnce(f64, f64) -> bool,
{
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(comparison(left, right))),
        _ => Err(error_for(locale, "P1010", position, operator)),
    }
}

fn compile(source: &str) -> Result<(Vec<Stmt>, Locale), PadmaError> {
    let locale = Locale::from_source(source);
    let tokens = Lexer::new(source, locale).tokenize()?;
    let program = Parser::new(tokens, locale).parse()?;
    Ok((program, locale))
}

fn format_diagnostic(path: &str, source: &str, error: &PadmaError, locale: Locale) -> String {
    let line = source
        .lines()
        .nth(error.position.line.saturating_sub(1))
        .unwrap_or("");
    let gutter_width = error.position.line.to_string().len();
    let marker = " ".repeat(error.position.column.saturating_sub(1));
    let (label, point, hint_label) = match locale {
        Locale::Bangla => ("ত্রুটি", "এই স্থানে", "পরামর্শ"),
        Locale::English => ("error", "here", "help"),
    };
    let mut rendered = format!(
        "{label}[{}]: {}\n  --> {}:{}:{}\n   |\n{:>width$} | {}\n   | {}^ {point}\n",
        error.code,
        error.message,
        path,
        error.position.line,
        error.position.column,
        error.position.line,
        line,
        marker,
        width = gutter_width
    );
    if let Some(hint) = &error.hint {
        rendered.push_str(&format!("   = {hint_label}: {hint}\n"));
    }
    rendered
}

fn usage(locale: Locale) -> &'static str {
    match locale {
        Locale::Bangla => {
            "ব্যবহার: padma [file.pd] অথবা padma <run|check|ast> <file.pd>\n\nকমান্ড:\n  padma                 interactive shell চালু করুন\n  padma <file.pd>       Padma script চালান\n  padma --version       version দেখুন\n  padma --help          এই help দেখুন\n\nউদাহরণ:\n  padma examples/hello-bn.pd\n  padma check examples/hello-en.pd\n  padma ast examples/mixed.pd\n"
        }
        Locale::English => {
            "Usage: padma [file.pd] or padma <run|check|ast> <file.pd>\n\nCommands:\n  padma                 open the interactive shell\n  padma <file.pd>       run a Padma script\n  padma --version       show the installed version\n  padma --help          show this help\n\nExamples:\n  padma examples/hello-en.pd\n  padma check examples/hello-en.pd\n  padma ast examples/mixed.pd\n"
        }
    }
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() == 1 {
        repl();
        return;
    }
    if arguments.len() == 2 && matches!(arguments[1].as_str(), "--version" | "-V") {
        println!("padma 0.1.0");
        return;
    }
    if arguments.len() == 2 && matches!(arguments[1].as_str(), "--help" | "-h") {
        println!("{}", usage(Locale::English));
        return;
    }
    let (command, path) = match arguments.len() {
        2 if arguments[1].ends_with(".pd") => ("run", arguments[1].as_str()),
        3 => (arguments[1].as_str(), arguments[2].as_str()),
        _ => {
            eprintln!("{}", usage(Locale::Bangla));
            process::exit(64);
        }
    };
    if path.is_empty() {
        eprintln!("{}", usage(Locale::Bangla));
        process::exit(64);
    }
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Cannot read `{path}`: {error}");
            process::exit(66);
        }
    };
    let locale = Locale::from_source(&source);
    let compiled = compile(&source);
    let (program, locale) = match compiled {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{}", format_diagnostic(path, &source, &error, locale));
            process::exit(1);
        }
    };

    match command {
        "check" => match locale {
            Locale::Bangla => println!("ঠিক আছে: `{path}`-এ কোনো syntax error পাওয়া যায়নি।"),
            Locale::English => println!("ok: no syntax errors found in `{path}`."),
        },
        "ast" => println!("{program:#?}"),
        "run" => {
            let mut interpreter = Interpreter::new(locale);
            if let Err(error) = interpreter.run(&program) {
                eprintln!("{}", format_diagnostic(path, &source, &error, locale));
                process::exit(1);
            }
            for line in interpreter.output {
                println!("{line}");
            }
        }
        _ => {
            eprintln!("{}", usage(locale));
            process::exit(64);
        }
    }
}

fn repl() {
    println!("Padma 0.1.0 (Bangla-English hybrid programming language)");
    println!("Interactive shell: help, copyright, credits, license; exit with exit() or বের হও.");
    let stdin = io::stdin();
    let mut interpreter = Interpreter::new(Locale::Bangla);
    loop {
        print!("padma> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let line = line.trim_end();
        if matches!(line, "exit" | "exit()" | "quit" | "quit()" | "বের হও") {
            break;
        }
        match line {
            "help" | "help()" | "সাহায্য" => {
                println!("Padma interactive shell commands:");
                println!("  help / সাহায্য       show this help");
                println!("  copyright            show copyright information");
                println!("  credits              show project credits");
                println!("  license              show the MIT license notice");
                println!("  exit() / বের হও      leave the shell");
                println!("Examples: দেখাও ২ + ৩ | print \"hello\" | ধরি x = 10");
                continue;
            }
            "copyright" | "copyright()" => {
                println!("Copyright (c) 2026 OfficialBiohub and Padma contributors.");
                continue;
            }
            "credits" | "credits()" => {
                println!("Padma is an open-source Bangla-English language project by OfficialBiohub and its contributors.");
                continue;
            }
            "license" | "license()" => {
                println!("Padma is released under the MIT License. See LICENSE in the repository.");
                continue;
            }
            _ => {}
        }
        if line.trim().is_empty() {
            continue;
        }
        match compile(&format!("{line}\n")) {
            Ok((program, locale)) => {
                interpreter.locale = locale;
                interpreter.return_value = None;
                let output_start = interpreter.output.len();
                match interpreter.run(&program) {
                    Ok(()) => {
                        for output in &interpreter.output[output_start..] {
                            println!("{output}");
                        }
                    }
                    Err(error) => eprintln!("{}", error.message),
                }
            }
            Err(error) => eprintln!("{}", error.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(source: &str) -> Result<Vec<String>, PadmaError> {
        let (program, locale) = compile(source)?;
        let mut interpreter = Interpreter::new(locale);
        interpreter.run(&program)?;
        Ok(interpreter.output)
    }

    #[test]
    fn runs_bangla_program_with_bangla_digits_and_interpolation() {
        let output = run(
            "ধরি নাম = \"রাফি\"\nধরি নম্বর = ৭০ + ২৩\nযদি নম্বর >= 90 {\n  দেখাও \"{নাম}: {নম্বর}\"\n}\n",
        )
        .unwrap();
        assert_eq!(output, vec!["রাফি: 93"]);
    }

    #[test]
    fn runs_english_program() {
        let output = run("let score = 15 * 2\nif score == 30 {\n  print \"passed\"\n}\n").unwrap();
        assert_eq!(output, vec!["passed"]);
    }

    #[test]
    fn supports_mixed_keyword_style() {
        let output = run("ধরি price = 250\nif price > 200 {\n  দেখাও \"discount\"\n}\n").unwrap();
        assert_eq!(output, vec!["discount"]);
    }

    #[test]
    fn reports_missing_bangla_variable_in_bangla() {
        let error = run("ধরি নাম = \"রাফি\"\nদেখাও বয়স\n").unwrap_err();
        assert_eq!(error.code, "P1007");
        assert!(error.message.contains("কোনো variable পাওয়া যায়নি"));
    }

    #[test]
    fn reports_missing_english_variable_in_english() {
        let error = run("let name = \"Rafi\"\nprint age\n").unwrap_err();
        assert_eq!(error.code, "P1007");
        assert!(error.message.contains("Cannot find variable"));
    }

    #[test]
    fn prevents_division_by_zero() {
        let error = run("let total = 10 / 0\n").unwrap_err();
        assert_eq!(error.code, "P1011");
    }

    #[test]
    fn parses_else_block() {
        let output = run(
            "let score = 40\nif score >= 50 {\n print \"pass\"\n} else {\n print \"retry\"\n}\n",
        )
        .unwrap();
        assert_eq!(output, vec!["retry"]);
    }

    #[test]
    fn updates_variables_and_runs_bangla_while_loop() {
        let output = run("ধরি i = ০\nযতক্ষণ i < ৩ {\n দেখাও i\n i = i + ১\n}\n").unwrap();
        assert_eq!(output, vec!["0", "1", "2"]);
    }

    #[test]
    fn reports_undefined_assignment_target() {
        let error = run("count = 1\n").unwrap_err();
        assert_eq!(error.code, "P1007");
    }

    #[test]
    fn stops_non_terminating_loop_with_safety_error() {
        let error = run("let i = 0\nwhile true {\n i = i + 1\n}\n").unwrap_err();
        assert_eq!(error.code, "P1012");
    }

    #[test]
    fn calls_function_with_parameters_and_return_value() {
        let output =
            run("function add(a, b) {\n return a + b\n}\nlet result = add(2, 3)\nprint result\n")
                .unwrap();
        assert_eq!(output, vec!["5"]);
    }

    #[test]
    fn evaluates_bengali_list_literal() {
        let output = run("ধরি সংখ্যা = [১, ২, ৩]\nদেখাও সংখ্যা\n").unwrap();
        assert_eq!(output, vec!["[1, 2, 3]"]);
    }

    #[test]
    fn creates_reads_and_updates_english_map() {
        let output = run(
            "let profile = {\"name\": \"Rafi\", \"age\": 12}\nprint profile.get(\"name\")\nprofile.set(\"age\", 13)\nprint profile.get(\"age\")\n",
        )
        .unwrap();
        assert_eq!(output, vec!["Rafi", "13"]);
    }

    #[test]
    fn creates_reads_and_updates_bangla_map() {
        let output = run(
            "ধরি প্রোফাইল = {\"নাম\": \"রাফি\", \"শ্রেণি\": ৬}\nদেখাও প্রোফাইল.get(\"নাম\")\nপ্রোফাইল.set(\"শ্রেণি\", ৭)\nদেখাও প্রোফাইল.get(\"শ্রেণি\")\n",
        )
        .unwrap();
        assert_eq!(output, vec!["রাফি", "7"]);
    }

    #[test]
    fn reports_missing_map_key() {
        let error =
            run("let profile = {\"name\": \"Rafi\"}\nprint profile.get(\"age\")\n").unwrap_err();
        assert_eq!(error.code, "P1021");
    }

    #[test]
    fn rejects_non_string_map_key() {
        let error = run("let profile = {\"name\": \"Rafi\"}\nprint profile.get(1)\n").unwrap_err();
        assert_eq!(error.code, "P1020");
    }
}
