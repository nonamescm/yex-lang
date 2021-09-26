#[derive(Debug, PartialEq)]
pub enum TokenType {
    Num(f64),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Lparen,
    Rparen,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let res = match self {
            Self::Num(n) => n.to_string(),
            Self::Add => '+'.into(),
            Self::Sub => '-'.into(),
            Self::Mul => '*'.into(),
            Self::Div => '/'.into(),
            Self::Neg => '~'.into(),
            Self::Lparen => '('.into(),
            Self::Rparen => ')'.into(),
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

