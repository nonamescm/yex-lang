#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Literals
    Num(f64),
    Str(String),
    Sym(vm::Symbol),
    Name(String),
    True,
    False,
    Nil,

    // Keywords
    If,
    Else,
    Then,
    Let,
    In,
    Fn,
    Become,
    Open,

    // logical operators
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Not,
    Assign,
    Cons,
    Len,

    // bitwise
    BitOr,
    BitAnd,
    BitXor,
    Shr, // right-shift
    Shl, // left-shift

    // Symbol
    Lparen,
    Rparen,
    Lbrack,
    Rbrack,
    Lbrace,
    Rbrace,
    Colon,
    Seq,
    Pipe,

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
            Self::Sym(s) => format!(":{}", s),
            Self::Name(v) => v.into(),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Nil => "nil".into(),

            Self::If => "if".into(),
            Self::Else => "else".into(),
            Self::Then => "then".into(),
            Self::Let => "let".into(),
            Self::Fn => "fn".into(),
            Self::In => "in".into(),
            Self::Become => "become".into(),
            Self::Open => "open".into(),

            Self::Add => '+'.into(),
            Self::Sub => '-'.into(),
            Self::Mul => '*'.into(),
            Self::Div => '/'.into(),
            Self::Eq => "==".into(),
            Self::Not => '~'.into(),
            Self::Assign => '='.into(),
            Self::Cons => "::".into(),
            Self::Len => '#'.into(),

            Self::BitAnd => "&&&".into(),
            Self::BitOr => "|||".into(),
            Self::BitXor => "^^^".into(),
            Self::Shr => ">>>".into(),
            Self::Shl => "<<<".into(),

            Self::Lparen => '('.into(),
            Self::Rparen => ')'.into(),
            Self::Lbrack => '['.into(),
            Self::Rbrack => ']'.into(),
            Self::Lbrace => '{'.into(),
            Self::Rbrace => '}'.into(),
            Self::Colon => ','.into(),
            Self::Seq => ">>".into(),
            Self::Pipe => "|>".into(),

            Self::Eof => "<eof>".into(),
        };

        write!(f, "{}", res)
    }
}

pub fn fetch_keyword<T: AsRef<str>>(word: T) -> Option<TokenType> {
    match word.as_ref() {
        "if" => Some(TokenType::If),
        "else" => Some(TokenType::Else),
        "then" => Some(TokenType::Then),
        "let" => Some(TokenType::Let),
        "in" => Some(TokenType::In),
        "true" => Some(TokenType::True),
        "false" => Some(TokenType::False),
        "nil" => Some(TokenType::Nil),
        "fn" => Some(TokenType::Fn),
        "become" => Some(TokenType::Become),
        "open" => Some(TokenType::Open),
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
