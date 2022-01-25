#[cfg(test)]
use vm::{gc::GcRef, Constant, List, OpCode, OpCodeMetadata, Symbol};

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
    use OpCode::*;

    let bytecode =
        compile("def _ = let oi = (((10))) in oi * 20").expect("Should be a valid syntax");
    let oi = 0;

    assert_eq!(
        bytecode,
        (
            vec![
                OpCodeMetadata {
                    line: 1,
                    column: 22,
                    opcode: Push(0)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 28,
                    opcode: Save(oi)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 31,
                    opcode: Load(oi)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 36,
                    opcode: Push(1)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 37,
                    opcode: Mul
                },
                OpCodeMetadata {
                    line: 1,
                    column: 37,
                    opcode: Drop(oi)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 37,
                    opcode: Savg(Symbol::new("_"))
                }
            ],
            vec![Constant::Num(10.0), Constant::Num(20.0)]
        )
    );

    assert_eq!(
        compile("def _ = [1, 2, 3]").unwrap(),
        (
            vec![
                OpCodeMetadata {
                    line: 1,
                    column: 9,
                    opcode: Push(0)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 16,
                    opcode: Push(3)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 17,
                    opcode: Prep
                },
                OpCodeMetadata {
                    line: 1,
                    column: 13,
                    opcode: Push(2)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 14,
                    opcode: Prep
                },
                OpCodeMetadata {
                    line: 1,
                    column: 10,
                    opcode: Push(1)
                },
                OpCodeMetadata {
                    line: 1,
                    column: 11,
                    opcode: Prep
                },
                OpCodeMetadata {
                    line: 1,
                    column: 18,
                    opcode: Savg(Symbol::new("_"))
                }
            ],
            vec![
                Constant::List(GcRef::new(List::new())),
                Constant::Num(1.0),
                Constant::Num(2.0),
                Constant::Num(3.0)
            ]
        )
    )
}
