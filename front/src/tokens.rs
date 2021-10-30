#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Literals
    Num(f64),
    Str(String),
    Sym(String),
    Var(String),
    True,
    False,
    Nil,

    // Keywords
    If,
    Else,
    Do,
    End,
    Val,
    In,

    // logical operators
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Not,
    Assign,

    // bitwise
    BitOr,
    BitAnd,
    BitXor,
    Shr, // right-shift
    Shl, // left-shift
    Lparen,
    Rparen,

    // Symbol
    Semicolon,

    Eof,
}

impl Default for TokenType {
    fn default() -> Self {
        Self::Eof
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::Num(n) => n.to_string(),
            Self::Str(s) => "\"".to_owned() + s + "\"",
            Self::Sym(s) => ":".to_owned() + s,
            Self::Var(v) => v.into(),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Nil => "nil".into(),

            Self::If => "if".into(),
            Self::Else => "else".into(),
            Self::Do => "do".into(),
            Self::End => "end".into(),
            Self::Val => "val".into(),
            Self::In => "in".into(),

            Self::Add => '+'.into(),
            Self::Sub => '-'.into(),
            Self::Mul => '*'.into(),
            Self::Div => '/'.into(),
            Self::Eq => "==".into(),
            Self::Not => '~'.into(),
            Self::Assign => '='.into(),
            Self::BitAnd => "&&&".into(),
            Self::BitOr => "|||".into(),
            Self::BitXor => "^^^".into(),
            Self::Shr => ">>>".into(),
            Self::Shl => "<<<".into(),
            Self::Lparen => '('.into(),
            Self::Rparen => ')'.into(),
            Self::Semicolon => ';'.into(),
            Self::Eof => "<eof>".into(),
        };

        write!(f, "{}", res)
    }
}

pub fn fetch_keyword<T: AsRef<str>>(word: T) -> Option<TokenType> {
    match word.as_ref() {
        "if" => Some(TokenType::If),
        "else" => Some(TokenType::Else),
        "do" => Some(TokenType::Do),
        "end" => Some(TokenType::End),
        "val" => Some(TokenType::Val),
        "in" => Some(TokenType::In),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub token: TokenType,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            line: 0,
            column: 0,
            token: TokenType::Eof,
        }
    }
}
