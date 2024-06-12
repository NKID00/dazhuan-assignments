use std::fmt::Display;

use indoc::indoc;
use itertools::Itertools;
use leptos::*;
use shiyanyi::*;
use thiserror::Error;

use crate::linalg::Row;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PositionedChar {
    pub c: char,
    pub row: usize,
    pub col: usize,
}

#[derive(Error, Debug, Clone)]
pub enum PreprocessError {
    #[error("invalid character {c:?} at {row}:{col}")]
    InvalidChar { c: char, row: usize, col: usize },
    #[error("unexpected EOF at {row}:{col} inside block comment")]
    EofWhileBlockComment { row: usize, col: usize },
    #[error("nested block comment at {row}:{col} is not implemented")]
    NestedBlockComment { row: usize, col: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CommentState {
    None,
    /// a slash is met
    EnteringInlineOrBlock(PositionedChar),
    Inline,
    Block,
    /// an asterisk is met in block comment
    LeavingBlock,
}

pub fn preprocess(source: String) -> Result<Vec<PositionedChar>, PreprocessError> {
    let mut preprocessed = vec![];
    let mut row = 1;
    let mut col = 1;
    let mut spaced = true;
    let mut comment_state = CommentState::None;
    for c in source.chars() {
        match comment_state {
            CommentState::None => {
                if c == '/' {
                    comment_state =
                        CommentState::EnteringInlineOrBlock(PositionedChar { c, row, col });
                    col += 1;
                    continue;
                }
            }
            CommentState::EnteringInlineOrBlock(slash) => match c {
                '/' => {
                    comment_state = CommentState::Inline;
                    if !spaced {
                        preprocessed.push(PositionedChar {
                            c: ' ',
                            row: slash.row,
                            col: slash.col,
                        });
                        spaced = true;
                    }
                    col += 1;
                    continue;
                }
                '*' => {
                    comment_state = CommentState::Block;
                    if !spaced {
                        preprocessed.push(PositionedChar {
                            c: ' ',
                            row: slash.row,
                            col: slash.col,
                        });
                        spaced = true;
                    }
                    col += 1;
                    continue;
                }
                _ => {
                    comment_state = CommentState::None;
                    preprocessed.push(slash);
                    spaced = false;
                }
            },
            CommentState::Inline => match c {
                '\n' => {
                    comment_state = CommentState::None;
                    row += 1;
                    col = 1;
                    continue;
                }
                _ => continue,
            },
            CommentState::Block => match c {
                '*' => {
                    comment_state = CommentState::LeavingBlock;
                    col += 1;
                    continue;
                }
                '\n' => {
                    row += 1;
                    col = 1;
                    continue;
                }
                _ => {
                    col += 1;
                    continue;
                }
            },
            CommentState::LeavingBlock => match c {
                '/' => {
                    comment_state = CommentState::None;
                    col += 1;
                    continue;
                }
                '\n' => {
                    comment_state = CommentState::Block;
                    row += 1;
                    col = 1;
                    continue;
                }
                _ => {
                    comment_state = CommentState::Block;
                    col += 1;
                    continue;
                }
            },
        }
        match c {
            '\n' => {
                if !spaced {
                    preprocessed.push(PositionedChar { c: ' ', row, col });
                    spaced = true;
                }
                row += 1;
                col = 1;
            }
            ' ' => {
                if !spaced {
                    preprocessed.push(PositionedChar { c: ' ', row, col });
                    spaced = true;
                }
                col += 1;
            }
            c => {
                preprocessed.push(PositionedChar { c, row, col });
                spaced = false;
                col += 1;
            }
        }
    }
    if comment_state == CommentState::Block {
        return Err(PreprocessError::EofWhileBlockComment { row, col });
    }
    if let Some(PositionedChar { c: ' ', .. }) = preprocessed.last() {
        preprocessed.pop();
    }
    Ok(preprocessed)
}

#[test]
fn test_preprocess() {
    let source = indoc! {"
        i/**/n
        t
    "}
    .to_string();
    let preprocessed = preprocess(source).unwrap();
    assert_eq!(
        preprocessed,
        [
            PositionedChar {
                c: 'i',
                row: 1,
                col: 1
            },
            PositionedChar {
                c: ' ',
                row: 1,
                col: 2
            },
            PositionedChar {
                c: 'n',
                row: 1,
                col: 6
            },
            PositionedChar {
                c: ' ',
                row: 1,
                col: 7
            },
            PositionedChar {
                c: 't',
                row: 2,
                col: 1
            },
        ]
    );

    let source = indoc! {"
        i/* // */n//
        t
    "}
    .to_string();
    let preprocessed = preprocess(source).unwrap();
    assert_eq!(
        preprocessed,
        [
            PositionedChar {
                c: 'i',
                row: 1,
                col: 1
            },
            PositionedChar {
                c: ' ',
                row: 1,
                col: 2
            },
            PositionedChar {
                c: 'n',
                row: 1,
                col: 10
            },
            PositionedChar {
                c: ' ',
                row: 1,
                col: 11
            },
            PositionedChar {
                c: 't',
                row: 2,
                col: 1
            },
        ]
    );

    let source = indoc! {"
        i/* */// */n//
        t
    "}
    .to_string();
    let preprocessed = preprocess(source).unwrap();
    assert_eq!(
        preprocessed,
        [
            PositionedChar {
                c: 'i',
                row: 1,
                col: 1
            },
            PositionedChar {
                c: ' ',
                row: 1,
                col: 2
            },
            PositionedChar {
                c: 't',
                row: 2,
                col: 1
            },
        ]
    );
}

/// Token ::= Ident | Sym | Kw | Op
/// Ident ::= [_a-zA-Z][_a-zA-Z0-9]*
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub token: TokenValue,
    pub row: usize,
    pub col: usize,
    pub raw: String,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:?} at {}:{}",
            self.token, self.raw, self.row, self.col
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenValue {
    Ident(Ident),
    Sym(Sym),
    Kw(Kw),
    Op(Op),
    LiteralInt(LiteralInt),
}

impl Display for TokenValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::Ident(ident) => write!(f, "Ident({})", ident.name),
            TokenValue::Sym(sym) => write!(f, "Sym::{sym:?}"),
            TokenValue::Kw(kw) => write!(f, "Kw::{kw:?}"),
            TokenValue::Op(op) => write!(f, "Op::{op:?}"),
            TokenValue::LiteralInt(literal_int) => write!(f, "LiteralInt({})", literal_int.value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sym {
    /// '('
    LeftParen,
    /// ')'
    RightParen,
    /// '['
    LeftBracket,
    /// ']'
    RightBracket,
    /// '{'
    LeftBrace,
    /// '}'
    RightBrace,
    /// ','
    Comma,
    /// ';'
    Semicolon,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kw {
    If,
    Int,
    For,
    While,
    Do,
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Op {
    /// '+'
    Add,
    /// '-'
    Sub,
    /// '*'
    Mul,
    /// '/'
    Div,
    /// '%'
    Mod,
    /// '='
    Assign,
    /// '>'
    Gt,
    /// '<'
    Lt,
    /// '>='
    Ge,
    /// '<='
    Le,
    /// '=='
    Eq,
    /// '!='
    Ne,
    /// '!'
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralInt {
    pub value: String,
}

#[derive(Error, Debug, Clone)]
pub enum LexError {
    #[error("unexpected {c:?} at {row}:{col}")]
    UnexpectedChar { c: char, row: usize, col: usize },
    #[error("unexpected EOF at {row}:{col}")]
    UnexpectedEof { row: usize, col: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AutomataState {
    Start,
    Ident_,
    IdentOrKwIfOrInt1,
    IdentOrKwIf2,
    IdentOrKwInt2,
    IdentOrKwInt3,
    IdentOrKwFor1,
    IdentOrKwFor2,
    IdentOrKwFor3,
    IdentOrKwWhile1,
    IdentOrKwWhile2,
    IdentOrKwWhile3,
    IdentOrKwWhile4,
    IdentOrKwWhile5,
    IdentOrKwDo1,
    IdentOrKwDo2,
    IdentOrKwReturn1,
    IdentOrKwReturn2,
    IdentOrKwReturn3,
    IdentOrKwReturn4,
    IdentOrKwReturn5,
    IdentOrKwReturn6,
    IdentOrKwBreak1,
    IdentOrKwBreak2,
    IdentOrKwBreak3,
    IdentOrKwBreak4,
    IdentOrKwBreak5,
    IdentOrKwContinue1,
    IdentOrKwContinue2,
    IdentOrKwContinue3,
    IdentOrKwContinue4,
    IdentOrKwContinue5,
    IdentOrKwContinue6,
    IdentOrKwContinue7,
    IdentOrKwContinue8,
    OpAssignOrEq,
    OpGtOrGe,
    OpLtOrLe,
    OpNotOrNe,
    LiteralInt,
    LiteralZero,
}

pub fn lex(preprocessed: Vec<PositionedChar>) -> Result<Vec<Token>, LexError> {
    // TODO: rewrite this tedious (required by the assignment) with macros
    use AutomataState::*;
    if preprocessed.is_empty() {
        return Ok(vec![]);
    }
    let mut tokens = vec![];
    let mut state = Start;
    let mut current_token_row = 0;
    let mut current_token_col = 0;
    let mut current_token_raw = "".to_string();
    let mut preprocessed = preprocessed.into_iter();
    let mut pc = preprocessed.next().unwrap();
    let mut keep_char = true;
    loop {
        if !keep_char {
            pc = match preprocessed.next() {
                Some(pc) => pc,
                None => break,
            };
        }
        keep_char = false;
        state = match state {
            Start => {
                if pc.c == ' ' {
                    continue;
                }
                current_token_row = pc.row;
                current_token_col = pc.col;
                current_token_raw = pc.c.to_string();
                match pc.c {
                    '0' => LiteralZero,
                    '1'..='9' => LiteralInt,
                    c @ ('_' | 'a'..='z' | 'A'..='Z') => match c {
                        'i' => IdentOrKwIfOrInt1,
                        'f' => IdentOrKwFor1,
                        'w' => IdentOrKwWhile1,
                        'd' => IdentOrKwDo1,
                        'r' => IdentOrKwReturn1,
                        'b' => IdentOrKwBreak1,
                        'c' => IdentOrKwContinue1,
                        _ => Ident_,
                    },
                    c @ ('(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | '+' | '-' | '*' | '/'
                    | '%') => {
                        match c {
                            '(' => tokens.push(Token {
                                token: TokenValue::Sym(Sym::LeftParen),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            ')' => tokens.push(Token {
                                token: TokenValue::Sym(Sym::RightParen),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            '[' => tokens.push(Token {
                                token: TokenValue::Sym(Sym::LeftBracket),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            ']' => tokens.push(Token {
                                token: TokenValue::Sym(Sym::RightBracket),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            '{' => tokens.push(Token {
                                token: TokenValue::Sym(Sym::LeftBrace),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            '}' => tokens.push(Token {
                                token: TokenValue::Sym(Sym::RightBrace),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            ',' => tokens.push(Token {
                                token: TokenValue::Sym(Sym::Comma),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            ';' => tokens.push(Token {
                                token: TokenValue::Sym(Sym::Semicolon),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            '+' => tokens.push(Token {
                                token: TokenValue::Op(Op::Add),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            '-' => tokens.push(Token {
                                token: TokenValue::Op(Op::Sub),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            '*' => tokens.push(Token {
                                token: TokenValue::Op(Op::Mul),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            '/' => tokens.push(Token {
                                token: TokenValue::Op(Op::Div),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            '%' => tokens.push(Token {
                                token: TokenValue::Op(Op::Mod),
                                row: current_token_row,
                                col: current_token_col,
                                raw: current_token_raw.clone(),
                            }),
                            _ => unreachable!(),
                        }
                        Start
                    }
                    '=' => OpAssignOrEq,
                    '>' => OpGtOrGe,
                    '<' => OpLtOrLe,
                    '!' => OpNotOrNe,
                    c => Err(LexError::UnexpectedChar {
                        c,
                        row: pc.row,
                        col: pc.col,
                    })?,
                }
            }
            Ident_ => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwIfOrInt1 => match pc.c {
                'f' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwIf2
                }
                'n' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwInt2
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwIf2 => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Kw(Kw::If),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwInt2 => match pc.c {
                't' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwInt3
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwInt3 => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Kw(Kw::Int),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwFor1 => match pc.c {
                'o' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwFor2
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwFor2 => match pc.c {
                'r' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwFor3
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwFor3 => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Kw(Kw::For),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwWhile1 => match pc.c {
                'h' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwWhile2
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwWhile2 => match pc.c {
                'i' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwWhile3
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwWhile3 => match pc.c {
                'l' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwWhile4
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwWhile4 => match pc.c {
                'e' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwWhile5
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwWhile5 => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Kw(Kw::While),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwDo1 => match pc.c {
                'o' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwDo2
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwDo2 => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Kw(Kw::Do),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwReturn1 => match pc.c {
                'e' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwReturn2
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwReturn2 => match pc.c {
                't' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwReturn3
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwReturn3 => match pc.c {
                'u' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwReturn4
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwReturn4 => match pc.c {
                'r' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwReturn5
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwReturn5 => match pc.c {
                'n' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwReturn6
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwReturn6 => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Kw(Kw::Return),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwBreak1 => match pc.c {
                'r' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwBreak2
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwBreak2 => match pc.c {
                'e' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwBreak3
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwBreak3 => match pc.c {
                'a' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwBreak4
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwBreak4 => match pc.c {
                'k' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwBreak5
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwBreak5 => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Kw(Kw::Break),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwContinue1 => match pc.c {
                'o' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwContinue2
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwContinue2 => match pc.c {
                'n' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwContinue3
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwContinue3 => match pc.c {
                't' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwContinue4
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwContinue4 => match pc.c {
                'i' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwContinue5
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwContinue5 => match pc.c {
                'n' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwContinue6
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwContinue6 => match pc.c {
                'u' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwContinue7
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwContinue7 => match pc.c {
                'e' => {
                    current_token_raw.push(pc.c);
                    IdentOrKwContinue8
                }
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Ident(Ident {
                            name: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            IdentOrKwContinue8 => match pc.c {
                '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    current_token_raw.push(pc.c);
                    Ident_
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Kw(Kw::Continue),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            OpAssignOrEq => match pc.c {
                '=' => {
                    current_token_raw.push(pc.c);
                    tokens.push(Token {
                        token: TokenValue::Op(Op::Eq),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    Start
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Op(Op::Assign),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            OpGtOrGe => match pc.c {
                '=' => {
                    current_token_raw.push(pc.c);
                    tokens.push(Token {
                        token: TokenValue::Op(Op::Ge),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    Start
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Op(Op::Gt),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            OpLtOrLe => match pc.c {
                '=' => {
                    current_token_raw.push(pc.c);
                    tokens.push(Token {
                        token: TokenValue::Op(Op::Le),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    Start
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Op(Op::Lt),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            OpNotOrNe => match pc.c {
                '=' => {
                    current_token_raw.push(pc.c);
                    tokens.push(Token {
                        token: TokenValue::Op(Op::Ne),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    Start
                }
                _ => {
                    tokens.push(Token {
                        token: TokenValue::Op(Op::Not),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            LiteralInt => match pc.c {
                c @ '0'..='9' => {
                    current_token_raw.push(c);
                    LiteralInt
                }
                c @ ('_' | 'a'..='z' | 'A'..='Z') => Err(LexError::UnexpectedChar {
                    c,
                    row: pc.row,
                    col: pc.col,
                })?,
                _ => {
                    tokens.push(Token {
                        token: TokenValue::LiteralInt(super::LiteralInt {
                            value: current_token_raw.clone(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
            LiteralZero => match pc.c {
                c @ ('_' | 'a'..='z' | 'A'..='Z' | '0'..='9') => Err(LexError::UnexpectedChar {
                    c,
                    row: pc.row,
                    col: pc.col,
                })?,
                _ => {
                    tokens.push(Token {
                        token: TokenValue::LiteralInt(super::LiteralInt {
                            value: "0".to_string(),
                        }),
                        row: current_token_row,
                        col: current_token_col,
                        raw: current_token_raw.clone(),
                    });
                    keep_char = true;
                    Start
                }
            },
        }
    }
    // handle EOF
    match state {
        Start => {}
        Ident_ => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwIfOrInt1 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwIf2 => tokens.push(Token {
            token: TokenValue::Kw(Kw::If),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwInt2 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwInt3 => tokens.push(Token {
            token: TokenValue::Kw(Kw::Int),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwFor1 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwFor2 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwFor3 => tokens.push(Token {
            token: TokenValue::Kw(Kw::For),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwWhile1 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwWhile2 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwWhile3 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwWhile4 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwWhile5 => tokens.push(Token {
            token: TokenValue::Kw(Kw::While),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwDo1 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwDo2 => tokens.push(Token {
            token: TokenValue::Kw(Kw::Do),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwReturn1 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwReturn2 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwReturn3 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwReturn4 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwReturn5 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwReturn6 => tokens.push(Token {
            token: TokenValue::Kw(Kw::Return),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwBreak1 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwBreak2 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwBreak3 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwBreak4 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwBreak5 => tokens.push(Token {
            token: TokenValue::Kw(Kw::Break),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwContinue1 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwContinue2 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwContinue3 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwContinue4 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwContinue5 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwContinue6 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwContinue7 => tokens.push(Token {
            token: TokenValue::Ident(Ident {
                name: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        IdentOrKwContinue8 => tokens.push(Token {
            token: TokenValue::Kw(Kw::Continue),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        OpAssignOrEq => tokens.push(Token {
            token: TokenValue::Op(Op::Assign),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        OpGtOrGe => tokens.push(Token {
            token: TokenValue::Op(Op::Gt),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        OpLtOrLe => tokens.push(Token {
            token: TokenValue::Op(Op::Lt),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        OpNotOrNe => tokens.push(Token {
            token: TokenValue::Op(Op::Not),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        LiteralInt => tokens.push(Token {
            token: TokenValue::LiteralInt(super::LiteralInt {
                value: current_token_raw.clone(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
        LiteralZero => tokens.push(Token {
            token: TokenValue::LiteralInt(super::LiteralInt {
                value: "0".to_string(),
            }),
            row: current_token_row,
            col: current_token_col,
            raw: current_token_raw.clone(),
        }),
    }
    Ok(tokens)
}

#[test]
fn test_lex() {
    let source = indoc! {"
        a/**/b
    "}
    .to_string();
    let preprocessed = preprocess(source).unwrap();
    assert_eq!(
        preprocessed,
        [
            PositionedChar {
                c: 'a',
                row: 1,
                col: 1
            },
            PositionedChar {
                c: ' ',
                row: 1,
                col: 2
            },
            PositionedChar {
                c: 'b',
                row: 1,
                col: 6
            },
        ]
    );
    let tokens = lex(preprocessed).unwrap();
    assert_eq!(
        tokens,
        [
            Token {
                token: TokenValue::Ident(Ident {
                    name: "a".to_string()
                }),
                row: 1,
                col: 1,
                raw: "a".to_string()
            },
            Token {
                token: TokenValue::Ident(Ident {
                    name: "b".to_string()
                }),
                row: 1,
                col: 6,
                raw: "b".to_string()
            },
        ]
    );
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LexerSolver;

impl Solver for LexerSolver {
    fn id(&self) -> String {
        "lexer".to_string()
    }

    fn title(&self) -> String {
        "词法分析器的构造".to_string()
    }

    fn description(&self) -> View {
        "输入类 C 语言源程序.".into_view()
    }

    fn default_input(&self) -> String {
        indoc! {"
            main()
            {
                int a, b;
                a = 10;
                b = a + 20;
            }
        "}
        .to_string()
    }

    fn solve(&self, input: String) -> View {
        let preprocessed = match preprocess(input) {
            Ok(preprocessed) => preprocessed,
            Err(e) => {
                return view! {
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "预处理" </p>
                        <pre class="text-red-500"> { e.to_string() } </pre>
                    </div>
                }
                .into_view()
            }
        };
        let preprocessed_string: String = preprocessed.iter().map(|pc| pc.c).collect();
        let tokens = match lex(preprocessed) {
            Ok(tokens) => tokens,
            Err(e) => {
                return view! {
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "预处理" </p>
                        <pre> { preprocessed_string } </pre>
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "词法分析" </p>
                        <pre class="text-red-500"> { e.to_string() } </pre>
                    </div>
                }
                .into_view()
            }
        };
        let tokens_string = tokens.iter().map(|token| token.to_string()).join("\n");
        view! {
            <div class="mb-10">
                <p class="font-bold mb-2"> "预处理" </p>
                <pre> { preprocessed_string } </pre>
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "词法分析" </p>
                <pre> { tokens_string } </pre>
            </div>
        }
        .into_view()
    }
}
