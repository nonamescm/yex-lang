#[derive(Debug, PartialEq)]
pub enum TokenType {
    Num(f64),
    Add,
    Sub,
    Mul,
    Div,
    Lparen,
    Rparen,

    Eof,
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
            Self::Eof => "<eof>".into(),
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
