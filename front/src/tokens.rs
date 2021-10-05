#[derive(Debug, PartialEq)]
pub enum TokenType {
    Num(f64),
    Add,
    Sub,
    Mul,
    Div,
    Lparen,
    Rparen,

    // Reserved for parse errors
    Err,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let res = match self {
            Self::Num(n) => n.to_string(),
            Self::Add => '+'.into(),
            Self::Sub => '-'.into(),
            Self::Mul => '*'.into(),
            Self::Div => '/'.into(),
            Self::Lparen => '('.into(),
            Self::Rparen => ')'.into(),
            Self::Err => unreachable!(),
        };

        write!(f, "{}", res)
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub token: TokenType
}

impl Default for Token {
    fn default() -> Self {
        Self {
            line: 0,
            column: 0,
            token: TokenType::Err,
        }
    }
}

impl Default for &Token {
    fn default() -> Self {
        &Token {
            line: 0,
            column: 0,
            token: TokenType::Err,
        }
    }
}
