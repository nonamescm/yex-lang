use vm::Symbol;

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

    fn throw<A, T: Into<String>>(&self, str: T) -> Result<A, ParseError> {
        ParseError::throw(self.line, self.column, str.into())
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

    fn back(&mut self) -> char {
        self.idx -= 1;
        self.current()
    }

    fn take_while(&mut self, cond: fn(char) -> bool) -> Result<String, ParseError> {
        let mut item = String::from(self.get_char(self.idx));

        while cond(self.get_char(self.idx + 1)) {
            if self.get_char(self.idx + 1) == '\0' {
                self.throw("Unclosed delimiter opened here")?;
            }
            self.next();
            item.push(self.current());
        }
        Ok(item)
    }

    fn take_unicode(&mut self, len: usize) -> Result<char, ParseError> {
        self.next();
        let mut unicode = String::new();
        while unicode.len() < len {
            if !self.current().is_ascii_hexdigit() {
                return self.throw("malformed Unicode character escape sequence");
            }
            unicode.push(self.current());
            self.next();
        }
        self.back();
        let unicode = u32::from_str_radix(&unicode, 16).unwrap();
        let unicode = char::from_u32(unicode).unwrap();
        Ok(unicode)
    }

    fn escape_char(&mut self) -> Result<String, ParseError> {
        let char = match self.current() {
            'n' => '\n',
            't' => '\t',
            'u' => self.take_unicode(4)?,
            'x' => self.take_unicode(2)?,
            'U' => self.take_unicode(8)?,
            '0' => EOF,
            '\\' => '\\',
            '"' => '"',
            'r' => '\r',
            other => self.throw(format!("Unknow escape char `{}`", other))?,
        };
        self.next();
        Ok(char.into())
    }

    fn take_str(&mut self) -> Result<String, ParseError> {
        let mut item = String::new();

        while self.current() != '"' {
            let chr = match self.current() {
                '\\' => {
                    self.next();
                    self.escape_char()?
                }
                EOF => self.throw("Unclosed delimiter opened here")?,
                other => {
                    let other = other.to_string();
                    self.next();
                    other
                }
            };

            item.push_str(&chr);
        }
        self.back();
        Ok(item)
    }

    fn peek_at(&self, n: usize) -> char {
        *self.tokens.get(self.idx + n).unwrap_or(&EOF)
    }

    fn get(&mut self) -> Tk {
        
        let tk = match self.current() {
            // comments
            '/' if self.peek_at(1) == '/' => {
                while !matches!(self.current(), '\n' | EOF) {
                    self.next();
                }
                return self.get();
            }

            '+' => TokenType::Add,
            '#' => TokenType::Len,
            '-' if self.peek_at(1) == '>' => {
                self.next();
                TokenType::Arrow
            }
            '-' => TokenType::Sub,
            '/' => TokenType::Div,
            '*' => TokenType::Mul,
            '%' => TokenType::Rem,
            '=' if self.peek_at(1) == '=' => {
                self.next();
                TokenType::Eq
            }
            '=' if self.peek_at(1) == '>' => {
                self.next();
                TokenType::FatArrow
            }
            '!' if self.peek_at(1) == '=' => {
                self.next();
                TokenType::Ne
            }

            '|' if self.peek_at(1) == '>' => {
                self.next();
                TokenType::Pipe
            }

            '(' => TokenType::Lparen,
            ')' => TokenType::Rparen,
            '[' => TokenType::Lbrack,
            ']' => TokenType::Rbrack,
            '{' => TokenType::Lbrace,
            '}' => TokenType::Rbrace,
            ':' if self.peek_at(1) == ':' => {
                self.next();
                TokenType::Cons
            }
            ':' if !self.peek_at(1).is_whitespace() => {
                self.next();
                let sym = self.take_while(|c| c.is_alphanumeric() || c == '_')?;
               
                match sym.as_str() {
                    "true" => TokenType::True,
                    "false" => TokenType::False,
                    "nil" => TokenType::Nil,
                    "\0" => self.throw("expected symbol string after `:`, found <eof>")?,
                    _ => TokenType::Sym(Symbol::new(sym)),
                }
            }
            '=' => TokenType::Assign,
            '"' if self.peek_at(1) == '"' => {
                self.next();
                TokenType::Str(String::new())
            }
            '"' => {
                self.next();
                let a = TokenType::Str(self.take_str()?);
                self.next();
                a
            }
            c if c.is_numeric() => {
                let n = self.take_while(|c| c.is_numeric() || c == '.')?;
                match n.parse::<f64>() {
                    Ok(n) => TokenType::Num(n),
                    Err(_) => self.throw(format!("Can't parse number {}", n))?,
                }
            }
            c if c.is_alphabetic() || c == '_' => {
                
                let mut tk = self.take_while(|c| c.is_alphanumeric() || c == '_')?;
                while matches!(self.peek_at(1), '?' | '!' | '\'') {
                    self.next();
                    tk.push(self.current());
                }
                
                if let Some(tk) = fetch_keyword(&tk) {
                    tk
                } else {
                    TokenType::Name(Symbol::new(tk))
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
                TokenType::Shl
            }

            '>' if self.peek_at(1) == '>' => {
                self.next();
                TokenType::Seq
            }

            '<' if self.peek_at(1) == '<' && self.peek_at(2) == '<' => {
                self.next();
                self.next();
                TokenType::Shr
            }
            '^' if self.peek_at(1) == '^' && self.peek_at(2) == '^' => {
                self.next();
                self.next();
                TokenType::BitXor
            }
            ',' => TokenType::Comma,
            ';' => TokenType::Semicolon,
            '<' if self.peek_at(1) == '=' => {
                self.next();
                TokenType::LessEq
            }
            '<' => TokenType::Less,
            '>' if self.peek_at(1) == '=' => {
                self.next();
                TokenType::GreaterEq
            }
            '>' => TokenType::Greater,
            '.' => TokenType::Dot,
            EOF => TokenType::Eof,

            c if c.is_whitespace() => {
                self.next();
                return self.get();
            }

            c => self.throw(format!("Unknown start of token `{}`", c))?,
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
