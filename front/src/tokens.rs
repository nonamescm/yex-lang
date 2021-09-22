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

#[derive(Debug, PartialEq)]
pub struct Token {
    pub line: usize,
    pub column: usize,
    pub token: TokenType
}
