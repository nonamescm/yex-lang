use crate::error::ParseError;
use crate::tokens::{Token, TokenType};

pub struct Lexer {
    line: usize,
    column: usize,
    tokens: Vec<char>,
    idx: usize,
}

type Tk = Result<Token, ParseError>;

impl Lexer {
    pub fn lex(tokens: Vec<char>) -> Vec<Tk> {
        let mut this = Self {
            line: 1,
            column: 1,
            tokens,
            idx: 0,
        };

        let mut reslt = Vec::new();

        while this.current() != '\0' {
            reslt.push(this.get());
            this.next();
        }

        reslt
    }

    fn get_char(&self, idx: usize) -> char {
        *self.tokens.get(idx).unwrap_or(&'\0')
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
            c if c.is_whitespace() => {
                self.column += 1;
                self.next();
            }
            '#' => {
                drop(self.take_while(|c| c != '\n'));
            }
            _ => self.column += 1,
        };
        self.current()
    }

    fn take_while(&mut self, cond: fn(char) -> bool) -> String {
        let mut item = String::from(self.tokens[self.idx]);

        while cond(self.get_char(self.idx + 1)) {
            self.next();
            item.push(self.current());
        }
        item
    }

    pub fn get(&mut self) -> Tk {
        let tk = match self.current() {
            '+' => TokenType::Add,
            '-' => TokenType::Sub,
            '/' => TokenType::Div,
            '*' => TokenType::Mul,
            '(' => TokenType::Lparen,
            ')' => TokenType::Rparen,
            c if c.is_numeric() => match self
                .take_while(|c| c.is_numeric() || c == '.')
                .parse::<f64>()
            {
                Ok(n) => TokenType::Num(n),
                Err(_) => ParseError::throw(self.line, self.column, "Can't parse number".into())?,
            },
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
