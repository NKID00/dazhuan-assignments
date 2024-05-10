use indoc::indoc;
use itertools::Itertools;
use leptos::{html::S, *};
use shiyanyi::*;

pub struct Token {
    pub token: TokenValue,
    pub row: usize,
    pub col: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenValue {
    Ident(Ident),
    Sym(Sym),
    Kw(Kw),
    Op(Op),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sym {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kw {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Assign,
    Ge,
    Le,
    Gt,
    Lt,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedChar {
    pub c: char,
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommentState {
    None,
    /// a slash is met
    EnteringInlineOrBlock(PositionedChar),
    Inline,
    Block,
    /// an asterisk is met in block comment
    LeavingBlock,
}

fn preprocess(source: String) -> Vec<PositionedChar> {
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
                c => {
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
                c => {
                    continue;
                }
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
                c => {
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
                c => {
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
    if let Some(PositionedChar { c: ' ', .. }) = preprocessed.last() {
        preprocessed.pop();
    }
    preprocessed
}

#[test]
fn test_preprocess() {
    let source = indoc!{"
        i/**/n
        t
    "}.to_string();
    let preprocessed = preprocess(source);
    assert_eq!(preprocessed, [
        PositionedChar{c: 'i', row: 1, col: 1 },
        PositionedChar{c: ' ', row: 1, col: 2 },
        PositionedChar{c: 'n', row: 1, col: 6 },
        PositionedChar{c: ' ', row: 1, col: 7 },
        PositionedChar{c: 't', row: 2, col: 1 },
    ]);

    let source = indoc!{"
        i/* // */n//
        t
    "}.to_string();
    let preprocessed = preprocess(source);
    assert_eq!(preprocessed, [
        PositionedChar{c: 'i', row: 1, col: 1 },
        PositionedChar{c: ' ', row: 1, col: 2 },
        PositionedChar{c: 'n', row: 1, col: 10 },
        PositionedChar{c: ' ', row: 1, col: 11 },
        PositionedChar{c: 't', row: 2, col: 1 },
    ]);

    let source = indoc!{"
        i/* */// */n//
        t
    "}.to_string();
    let preprocessed = preprocess(source);
    assert_eq!(preprocessed, [
        PositionedChar{c: 'i', row: 1, col: 1 },
        PositionedChar{c: ' ', row: 1, col: 2 },
        PositionedChar{c: 't', row: 2, col: 1 },
    ]);
}

fn lex(preprocessed: Vec<PositionedChar>) -> Vec<Token> {
    vec![]
}

#[test]
fn test_lex() {}

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
        let preprocessed = preprocess(input);
        let preprocessed_string: String = preprocessed.iter().map(|pc| pc.c).collect();
        view! {
            <div class="mb-10">
                <p class="font-bold mb-2"> "预处理" </p>
                <pre> { preprocessed_string } </pre>
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "词法分析" </p>
                <pre> { "TODO" } </pre>
            </div>
        }
        .into_view()
    }
}
