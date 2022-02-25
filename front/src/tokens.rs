#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Literals
    Num(f64),
    Str(String),
    Sym(vm::Symbol),
    Name(vm::Symbol),
    True,
    False,
    Nil,

    // Keywords
    If,
    Else,
    Then,
    Loop,
    Def,
    Let,
    In,
    Fn,
    Become,
    Open,
    And,
    Or,
    Not,

    // logical operators
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Greater,
    GreaterEq,
    Less,
    LessEq,
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
    Comma,
    Semicolon,
    Seq,
    Pipe,
    Arrow,
    FatArrow,

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
            Self::Name(v) => format!("{}", v),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Nil => "nil".into(),

            Self::If => "if".into(),
            Self::Else => "else".into(),
            Self::Then => "then".into(),
            Self::Loop => "loop".into(),
            Self::Def => "def".into(),
            Self::Let => "let".into(),
            Self::Fn => "fn".into(),
            Self::In => "in".into(),
            Self::Become => "become".into(),
            Self::Open => "open".into(),
            Self::And => "and".into(),
            Self::Or => "or".into(),
            Self::Not => "not".into(),

            Self::Add => '+'.into(),
            Self::Sub => '-'.into(),
            Self::Mul => '*'.into(),
            Self::Div => '/'.into(),
            Self::Rem => '%'.into(),
            Self::Eq => "==".into(),
            Self::Ne => "!=".into(),
            Self::Greater => ">".into(),
            Self::GreaterEq => ">=".into(),
            Self::Less => "<".into(),
            Self::LessEq => "<=".into(),
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
            Self::Comma => ','.into(),
            Self::Semicolon => ';'.into(),
            Self::Seq => ">>".into(),
            Self::Pipe => "|>".into(),
            Self::Arrow => "->".into(),
            Self::FatArrow => "=>".into(),
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
        "loop" => Some(TokenType::Loop),
        "def" => Some(TokenType::Def),
        "let" => Some(TokenType::Let),
        "in" => Some(TokenType::In),
        "true" => Some(TokenType::True),
        "false" => Some(TokenType::False),
        "nil" => Some(TokenType::Nil),
        "fn" => Some(TokenType::Fn),
        "become" => Some(TokenType::Become),
        "open" => Some(TokenType::Open),
        "and" => Some(TokenType::And),
        "or" => Some(TokenType::Or),
        "not" => Some(TokenType::Not),
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
