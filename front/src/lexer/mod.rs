use crate::error::ParseError;
use crate::tokens::{fetch_keyword, Token, TokenType};

const EOF: char = '\0';

pub struct Lexer {
    line: usize,
    column: usize,
    tokens: Vec<char>,
    idx: usize,
}

type Tk = Result<Token, ParseError>;

impl Lexer {
    pub fn new<T: Into<String>>(t: T) -> Self {
        Self {
            tokens: t.into().chars().collect(),
            line: 1,
            column: 1,
            idx: 0,
        }
    }
    // Implemented for tests
    #[cfg(test)]
    pub fn lex(tokens: Vec<char>) -> Vec<Tk> {
        let mut this = Self {
            line: 1,
            column: 1,
            tokens,
            idx: 0,
        };

        let mut reslt = Vec::new();

        while this.current() != EOF {
            reslt.push(this.get());
            this.next();
        }

        reslt
    }

    fn get_char(&self, idx: usize) -> char {
        *self.tokens.get(idx).unwrap_or(&EOF)
    }

    fn current(&self) -> char {
        self.get_char(self.idx)
    }

    fn next(&mut self) -> char {
        self.idx += 1;
        match self.current() {
            '\n' => {
                self.column = 1;
                self.line += 1;
            }
            _ => self.column += 1,
        };
        self.current()
    }

    fn take_while(&mut self, cond: fn(char) -> bool) -> Result<String, ParseError> {
        let mut item = String::from(self.get_char(self.idx));

        while cond(self.get_char(self.idx + 1)) {
            if self.get_char(self.idx + 1) == '\0' {
                ParseError::throw(
                    self.line,
                    self.column,
                    "Unclosed delimiter opened here".into(),
                )?;
            }

            self.next();
            item.push(self.current());
        }
        Ok(item)
    }

    fn peek_at(&self, n: usize) -> char {
        *self.tokens.get(self.idx + n).unwrap_or(&EOF)
    }

    fn get(&mut self) -> Tk {
        let tk = match self.current() {
            '+' => TokenType::Add,
            '-' if self.peek_at(1) == '>' => {
                self.next();
                TokenType::Arrow
            }
            '-' => TokenType::Sub,
            '/' => TokenType::Div,
            '*' => TokenType::Mul,
            '=' if self.peek_at(1) == '=' => {
                self.next();
                TokenType::Eq
            }
            '(' => TokenType::Lparen,
            ')' => TokenType::Rparen,
            ':' if !self.peek_at(1).is_whitespace() => {
                self.next();
                let sym = self.take_while(|c| c.is_alphanumeric() || c == '_')?;
                match sym.as_str() {
                    "true" => TokenType::True,
                    "false" => TokenType::False,
                    "nil" => TokenType::Nil,
                    "\0" => ParseError::throw(
                        self.line,
                        self.column,
                        "expected symbol string after `:`, found <eof>".into(),
                    )?,
                    _ => TokenType::Sym(vm::Symbol::new(sym)),
                }
            }
            '=' => TokenType::Assign,
            '~' => TokenType::Not,
            '"' => {
                self.next();
                let a = TokenType::Str(self.take_while(|c| c != '"')?);
                self.next();
                a
            }
            c if c.is_numeric() => {
                let n = self.take_while(|c| c.is_numeric() || c == '.')?;
                match n.parse::<f64>() {
                    Ok(n) => TokenType::Num(n),
                    Err(_) => ParseError::throw(
                        self.line,
                        self.column,
                        format!("Can't parse number {}", n),
                    )?,
                }
            }
            c if c.is_alphabetic() || c == '_' => {
                let tk = self.take_while(|c| c.is_alphanumeric() || c == '_')?;
                if let Some(tk) = fetch_keyword(&tk) {
                    tk
                } else {
                    TokenType::Idnt(tk)
                }
            }

            // BitWise
            '&' if self.peek_at(1) == '&' && self.peek_at(2) == '&' => {
                self.next();
                self.next();
                TokenType::BitAnd
            }
            '|' if self.peek_at(1) == '|' && self.peek_at(2) == '|' => {
                self.next();
                self.next();
                TokenType::BitOr
            }
            '>' if self.peek_at(1) == '>' && self.peek_at(2) == '>' => {
                self.next();
                self.next();
                TokenType::Shr
            }
            '<' if self.peek_at(1) == '<' && self.peek_at(2) == '<' => {
                self.next();
                self.next();
                TokenType::Shl
            }
            '^' if self.peek_at(1) == '^' && self.peek_at(2) == '^' => {
                self.next();
                self.next();
                TokenType::BitXor
            }

            ';' => TokenType::Semicolon,

            EOF => TokenType::Eof,

            c if c.is_whitespace() => {
                self.next();
                return self.get();
            }

            c => ParseError::throw(
                self.line,
                self.column,
                format!("Unknown start of token `{}`", c),
            )?,
        };

        Ok(Token {
            line: self.line,
            column: self.column,
            token: tk,
        })
    }
}

impl Iterator for Lexer {
    type Item = Result<Token, ParseError>;
    fn next(&mut self) -> Option<Self::Item> {
        let x = self.get();
        self.next();
        Some(x)
    }
}
