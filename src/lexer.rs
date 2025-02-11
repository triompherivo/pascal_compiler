// src/lexer.rs

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Program,
    Var,
    Begin,
    End,
    If,
    Then,
    Else,
    While,
    Do,
    For,
    To,
    Downto,
    Repeat,
    Until,
    Case,
    Of,
    Function,
    Procedure,
    Writeln,
    Readln,
    Record, // new token for record literal construction
    // Operators and punctuation
    Plus,
    Minus,
    Star,
    Slash,
    ColonEqual,
    Equal,
    Colon,
    Semicolon,
    Comma,
    Dot,
    LParen,
    RParen,
    // Identifiers and literals
    Identifier(String),
    Number(String),
    StringLiteral(String),
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer { input: input.chars().collect(), pos: 0 }
    }
    
    fn current(&self) -> Option<char> {
        self.input.get(self.pos).cloned()
    }
    
    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        self.pos += 1;
        ch
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() { self.advance(); } else { break; }
        }
    }
    
    /// Skip a comment delimited by { and }.
    fn skip_comment(&mut self) {
        self.advance(); // skip '{'
        while let Some(c) = self.current() {
            if c == '}' {
                self.advance(); // skip '}'
                break;
            }
            self.advance();
        }
    }
    
    /// Lex a string literal delimited by double quotes.
    fn lex_string(&mut self) -> Token {
        self.advance(); // skip opening "
        let mut s = String::new();
        while let Some(c) = self.current() {
            if c == '"' {
                self.advance(); // skip closing "
                break;
            } else {
                s.push(c);
                self.advance();
            }
        }
        Token::StringLiteral(s)
    }
    
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        if let Some('{') = self.current() {
            self.skip_comment();
            self.skip_whitespace();
            return self.next_token();
        }
        
        if let Some('"') = self.current() {
            return self.lex_string();
        }
        
        let ch = match self.current() {
            Some(c) => c,
            None => return Token::EOF,
        };
        
        // Lex numbers (supporting an optional decimal point)
        if ch.is_digit(10) {
            let mut num = String::new();
            while let Some(c) = self.current() {
                if c.is_digit(10) {
                    num.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            if let Some('.') = self.current() {
                num.push('.');
                self.advance();
                while let Some(c) = self.current() {
                    if c.is_digit(10) {
                        num.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            return Token::Number(num);
        }
        
        // Lex identifiers and keywords.
        if ch.is_alphabetic() {
            let mut ident = String::new();
            while let Some(c) = self.current() {
                if c.is_alphanumeric() {
                    ident.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            match ident.to_lowercase().as_str() {
                "program"   => return Token::Program,
                "var"       => return Token::Var,
                "begin"     => return Token::Begin,
                "end"       => return Token::End,
                "if"        => return Token::If,
                "then"      => return Token::Then,
                "else"      => return Token::Else,
                "while"     => return Token::While,
                "do"        => return Token::Do,
                "for"       => return Token::For,
                "to"        => return Token::To,
                "downto"    => return Token::Downto,
                "repeat"    => return Token::Repeat,
                "until"     => return Token::Until,
                "case"      => return Token::Case,
                "of"        => return Token::Of,
                "function"  => return Token::Function,
                "procedure" => return Token::Procedure,
                "writeln"   => return Token::Writeln,
                "readln"    => return Token::Readln,
                "record"    => return Token::Record, // new
                _           => return Token::Identifier(ident),
            }
        }
        
        // Operators and punctuation.
        match ch {
            '+' => { self.advance(); return Token::Plus; },
            '-' => { self.advance(); return Token::Minus; },
            '*' => { self.advance(); return Token::Star; },
            '/' => { self.advance(); return Token::Slash; },
            ':' => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    return Token::ColonEqual;
                } else {
                    return Token::Colon;
                }
            },
            '=' => { self.advance(); return Token::Equal; },
            ';' => { self.advance(); return Token::Semicolon; },
            ',' => { self.advance(); return Token::Comma; },
            '.' => { self.advance(); return Token::Dot; },
            '(' => { self.advance(); return Token::LParen; },
            ')' => { self.advance(); return Token::RParen; },
            _ => { self.advance(); return Token::EOF; }
        }
    }
}
