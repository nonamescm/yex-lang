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
            Self::Colon => ','.into(),
            Self::Seq => ">>".into(),
            Self::Pipe => "|>".into(),
            Self::Eof => "<eof>".into(),
        };

        write!(f, "{}", res)
    }
}

impl TokenType {
    pub fn is_expression(&self) -> bool {
        let kw = matches!(
            self,
            Self::Eof
                | Self::Colon
                | Self::Rbrack
                | Self::Rparen
                | Self::Rbrace
                | Self::If
                | Self::Else
                | Self::Then
                | Self::Loop
                | Self::Def
                | Self::Let
                | Self::In
                | Self::Fn
                | Self::Become
                | Self::Open
                | Self::And
                | Self::Or
                | Self::Not
        );

        let op = self.is_binary_operator();

        !kw && !op
    }

    pub fn is_binary_operator(&self) -> bool {
        matches!(
            self,
            Self::Add
                | Self::Sub
                | Self::Mul
                | Self::Div
                | Self::Rem
                | Self::Eq
                | Self::Ne
                | Self::Greater
                | Self::GreaterEq
                | Self::Less
                | Self::LessEq
                | Self::Assign
                | Self::Cons
                | Self::Len
                | Self::Seq
                | Self::Pipe
                | Self::BitAnd
                | Self::BitOr
                | Self::BitXor
                | Self::Shr
                | Self::Shl
                | Self::And
                | Self::Or
        )
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
