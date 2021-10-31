#[cfg(test)]
use vm::{Bytecode, OpCodeMetadata, OpCode, Constant, Symbol};

#[test]
fn lex_test() {
    use crate::lexer::Lexer;
    use crate::tokens::{Token, TokenType::*};

    assert_eq!(
        Lexer::lex("1 + 1".chars().collect()),
        vec![
            Ok(Token {
                line: 1,
                column: 1,
                token: Num(1.0)
            }),
            Ok(Token {
                line: 1,
                column: 3,
                token: Add
            }),
            Ok(Token {
                line: 1,
                column: 5,
                token: Num(1.0)
            }),
        ]
    )
}

#[test]
fn lex_test_2() {
    use crate::lexer::Lexer;
    use crate::tokens::{Token, TokenType::*};

    assert_eq!(
        Lexer::lex("(1+1) * 2 - (2.2/3)".chars().collect()),
        vec![
            Ok(Token {
                line: 1,
                column: 1,
                token: Lparen
            }),
            Ok(Token {
                line: 1,
                column: 2,
                token: Num(1.0)
            }),
            Ok(Token {
                line: 1,
                column: 3,
                token: Add
            }),
            Ok(Token {
                line: 1,
                column: 4,
                token: Num(1.0)
            }),
            Ok(Token {
                line: 1,
                column: 5,
                token: Rparen
            }),
            Ok(Token {
                line: 1,
                column: 7,
                token: Mul
            }),
            Ok(Token {
                line: 1,
                column: 9,
                token: Num(2.0)
            }),
            Ok(Token {
                line: 1,
                column: 11,
                token: Sub
            }),
            Ok(Token {
                line: 1,
                column: 13,
                token: Lparen
            }),
            Ok(Token {
                line: 1,
                column: 16,
                token: Num(2.2)
            }),
            Ok(Token {
                line: 1,
                column: 17,
                token: Div
            }),
            Ok(Token {
                line: 1,
                column: 18,
                token: Num(3.0)
            }),
            Ok(Token {
                line: 1,
                column: 19,
                token: Rparen
            })
        ]
    )
}

#[test]
fn test_compiler() {
    use crate::compile;

    let bytecode = compile("val oi = 10 in oi * 20").expect("Should be a valid syntax");

    assert_eq!(
        bytecode,
        Bytecode {
            instructions: vec![
                OpCodeMetadata {
                    line: 1,
                    column: 11,
                    opcode: OpCode::Push(0)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 14,
                    opcode: OpCode::Save(1)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 17,
                    opcode: OpCode::Load(1)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 22,
                    opcode: OpCode::Push(2)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 23,
                    opcode: OpCode::Mul
                },
                OpCodeMetadata {
                    line: 1,
                    column: 23,
                    opcode: OpCode::Drop(1)
                }
            ],
            constants: vec![
                Constant::Num(10.0),
                Constant::Val(Symbol::new("oi")),
                Constant::Num(20.0)
            ]
        }
    );
}
