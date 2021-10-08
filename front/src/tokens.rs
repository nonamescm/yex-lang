#[derive(Debug, PartialEq)]
pub enum TokenType {
    Num(f64),
    Str(String),
    Sym(String),
    True,
    False,

    // logical operators
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Not,

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
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Add => '+'.into(),
            Self::Sub => '-'.into(),
            Self::Mul => '*'.into(),
            Self::Div => '/'.into(),
            Self::Eq => '='.into(),
            Self::Not => '~'.into(),
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
