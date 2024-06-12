use std::{collections::HashMap, fmt::Display, sync::OnceLock};

use indoc::indoc;
use itertools::Itertools;
use leptos::*;
use shiyanyi::*;
use thiserror::Error;

use super::{lex, preprocess, LiteralInt, Op, Sym, Token, TokenValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Term {
    Terminal(Terminal),
    Nonterminal(Nonterminal),
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Terminal(terminal) => write!(f, "{terminal}"),
            Term::Nonterminal(nonterminal) => write!(f, "{nonterminal}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Terminal {
    Ident,
    Sym(Sym),
    Op(Op),
    LiteralInt,
    /// End of input stream
    Eos,
}

impl Display for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Terminal::Ident => write!(f, "Ident"),
            Terminal::Sym(Sym::LeftBrace) => write!(f, "("),
            Terminal::Sym(Sym::RightBrace) => write!(f, ")"),
            Terminal::Op(Op::Add) => write!(f, "+"),
            Terminal::Op(Op::Mul) => write!(f, "*"),
            Terminal::LiteralInt => write!(f, "LiteralInt"),
            Terminal::Eos => write!(f, "#"),
            _ => write!(f, "Invalid"),
        }
    }
}

impl TryFrom<Token> for Terminal {
    type Error = Token;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token {
            TokenValue::Ident(_) => Ok(Self::Ident),
            TokenValue::Sym(Sym::LeftBrace) => Ok(Self::Sym(Sym::LeftBrace)),
            TokenValue::Sym(Sym::RightBrace) => Ok(Self::Sym(Sym::RightBrace)),
            TokenValue::Op(Op::Add) => Ok(Self::Op(Op::Add)),
            TokenValue::Op(Op::Mul) => Ok(Self::Op(Op::Mul)),
            TokenValue::LiteralInt(_) => Ok(Self::LiteralInt),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Nonterminal {
    name: String,
}

impl Display for Nonterminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LL1Rule {
    lhs: Nonterminal,
    rhs: Vec<Term>,
}

impl Display for LL1Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ::= ", self.lhs)?;
        if self.rhs.is_empty() {
            write!(f, "\\epsilon")
        } else {
            write!(f, "{}", self.rhs.iter().map(|t| t.to_string()).join(" "))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LL1ParseTable {
    start: Nonterminal,
    rules: Vec<LL1Rule>,
    table: HashMap<(Nonterminal, Terminal), usize>,
}

impl Default for LL1ParseTable {
    fn default() -> Self {
        static RULES: OnceLock<Vec<LL1Rule>> = OnceLock::new();
        static TABLE: OnceLock<HashMap<(Nonterminal, Terminal), usize>> = OnceLock::new();
        // Rules:
        // (0)  E  ::= T E'
        // (1)  E' ::= + T E'
        // (2)  E' ::= \epsilon
        // (3)  T  ::= F T'
        // (4)  T' ::= * F T'
        // (5)  T' ::= \epsilon
        // (6)  F  ::= ( E )
        // (7)  F  ::= Ident
        // (8)  F  ::= LiteralInt
        let rules = RULES
            .get_or_init(|| {
                vec![
                    // E  ::= T E'
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "E".to_string(),
                        },
                        rhs: vec![
                            Term::Nonterminal(Nonterminal {
                                name: "T".to_string(),
                            }),
                            Term::Nonterminal(Nonterminal {
                                name: "E'".to_string(),
                            }),
                        ],
                    },
                    // E' ::= + T E'
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "E'".to_string(),
                        },
                        rhs: vec![
                            Term::Terminal(Terminal::Op(Op::Add)),
                            Term::Nonterminal(Nonterminal {
                                name: "T".to_string(),
                            }),
                            Term::Nonterminal(Nonterminal {
                                name: "E'".to_string(),
                            }),
                        ],
                    },
                    // E' ::= \epsilon
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "E'".to_string(),
                        },
                        rhs: vec![],
                    },
                    // T  ::= F T'
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "T".to_string(),
                        },
                        rhs: vec![
                            Term::Nonterminal(Nonterminal {
                                name: "F".to_string(),
                            }),
                            Term::Nonterminal(Nonterminal {
                                name: "T'".to_string(),
                            }),
                        ],
                    },
                    // T' ::= * F T'
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "T'".to_string(),
                        },
                        rhs: vec![
                            Term::Terminal(Terminal::Op(Op::Mul)),
                            Term::Nonterminal(Nonterminal {
                                name: "F".to_string(),
                            }),
                            Term::Nonterminal(Nonterminal {
                                name: "T'".to_string(),
                            }),
                        ],
                    },
                    // T' ::= \epsilon
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "T'".to_string(),
                        },
                        rhs: vec![],
                    },
                    // F ::= ( E )
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "F".to_string(),
                        },
                        rhs: vec![
                            Term::Terminal(Terminal::Sym(Sym::LeftParen)),
                            Term::Nonterminal(Nonterminal {
                                name: "E".to_string(),
                            }),
                            Term::Terminal(Terminal::Sym(Sym::RightParen)),
                        ],
                    },
                    // F ::= Ident
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "F".to_string(),
                        },
                        rhs: vec![Term::Terminal(Terminal::Ident)],
                    },
                    // F ::= LiteralInt
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "F".to_string(),
                        },
                        rhs: vec![Term::Terminal(Terminal::LiteralInt)],
                    },
                ]
            })
            .clone();
        // Table:
        //         +       *       (       )       Ident       LiteralInt      #
        // E                       0               0           0
        // E'      1                       2                                   2
        // T                       3               3           3
        // T'      5       4               5                                   5
        // F                       6               7           8
        let table = TABLE
            .get_or_init(|| {
                HashMap::from([
                    // E ( 0
                    (
                        (
                            Nonterminal {
                                name: "E".to_string(),
                            },
                            Terminal::Sym(Sym::LeftBrace),
                        ),
                        0,
                    ),
                    // E Ident 0
                    (
                        (
                            Nonterminal {
                                name: "E".to_string(),
                            },
                            Terminal::Ident,
                        ),
                        0,
                    ),
                    // E LiteralInt 0
                    (
                        (
                            Nonterminal {
                                name: "E".to_string(),
                            },
                            Terminal::LiteralInt,
                        ),
                        0,
                    ),
                    // E' + 1
                    (
                        (
                            Nonterminal {
                                name: "E'".to_string(),
                            },
                            Terminal::Op(Op::Add),
                        ),
                        1,
                    ),
                    // E' ( 2
                    (
                        (
                            Nonterminal {
                                name: "E'".to_string(),
                            },
                            Terminal::Sym(Sym::RightBrace),
                        ),
                        2,
                    ),
                    // E' # 2
                    (
                        (
                            Nonterminal {
                                name: "E'".to_string(),
                            },
                            Terminal::Eos,
                        ),
                        2,
                    ),
                    // T ( 3
                    (
                        (
                            Nonterminal {
                                name: "T".to_string(),
                            },
                            Terminal::Sym(Sym::LeftBrace),
                        ),
                        3,
                    ),
                    // T Ident 3
                    (
                        (
                            Nonterminal {
                                name: "T".to_string(),
                            },
                            Terminal::Ident,
                        ),
                        3,
                    ),
                    // T LiteralInt 3
                    (
                        (
                            Nonterminal {
                                name: "T".to_string(),
                            },
                            Terminal::LiteralInt,
                        ),
                        3,
                    ),
                    // T' + 5
                    (
                        (
                            Nonterminal {
                                name: "T'".to_string(),
                            },
                            Terminal::Op(Op::Add),
                        ),
                        5,
                    ),
                    // T' * 4
                    (
                        (
                            Nonterminal {
                                name: "T'".to_string(),
                            },
                            Terminal::Op(Op::Mul),
                        ),
                        4,
                    ),
                    // T' ) 5
                    (
                        (
                            Nonterminal {
                                name: "T'".to_string(),
                            },
                            Terminal::Sym(Sym::RightBrace),
                        ),
                        5,
                    ),
                    // T' # 5
                    (
                        (
                            Nonterminal {
                                name: "T'".to_string(),
                            },
                            Terminal::Eos,
                        ),
                        5,
                    ),
                    // F ( 6
                    (
                        (
                            Nonterminal {
                                name: "F".to_string(),
                            },
                            Terminal::Sym(Sym::LeftBrace),
                        ),
                        6,
                    ),
                    // F Ident 7
                    (
                        (
                            Nonterminal {
                                name: "F".to_string(),
                            },
                            Terminal::Ident,
                        ),
                        7,
                    ),
                    // F LiteralInt 8
                    (
                        (
                            Nonterminal {
                                name: "F".to_string(),
                            },
                            Terminal::LiteralInt,
                        ),
                        8,
                    ),
                ])
            })
            .clone();
        Self {
            start: Nonterminal {
                name: "E".to_string(),
            },
            rules,
            table,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParseTrace {
    stack: Vec<Term>,
    input: Vec<Token>,
    rule: Option<usize>,
}

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("invalid token {token}")]
    InvalidToken { token: Token },
    #[error("unexpected {token}")]
    UnexpectedToken { token: Token },
    #[error("expect end of stream, found {token}")]
    ExtraToken { token: Token },
    #[error("unexpected end of stream")]
    UnexpectedEos,
}

fn parse(
    parse_table: LL1ParseTable,
    input: Vec<Token>,
) -> (Vec<ParseTrace>, Result<(), ParseError>) {
    let mut trace = vec![];
    let mut stack = vec![];
    stack.push(Term::Nonterminal(parse_table.start));
    let input: Result<Vec<(Token, Terminal)>, Token> = input
        .into_iter()
        .rev()
        .map(|token| token.clone().try_into().map(|terminal| (token, terminal)))
        .collect();
    let mut input = match input {
        Ok(input) => input,
        Err(token) => return (trace, Err(ParseError::InvalidToken { token })),
    };
    while !(stack.is_empty() && input.is_empty()) {
        match stack.last() {
            Some(Term::Terminal(terminal)) => match input.last() {
                Some((token, token_terminal)) => {
                    if terminal == token_terminal {
                        trace.push(ParseTrace {
                            stack: stack.clone(),
                            input: input.iter().map(|v| v.0.clone()).collect(),
                            rule: None,
                        });
                        stack.pop();
                        input.pop();
                    } else {
                        return (
                            trace,
                            Err(ParseError::UnexpectedToken {
                                token: token.clone(),
                            }),
                        );
                    }
                }
                None => return (trace, Err(ParseError::UnexpectedEos)),
            },
            Some(Term::Nonterminal(nonterminal)) => match input.last() {
                Some((token, token_terminal)) => {
                    let index = match parse_table
                        .table
                        .get(&(nonterminal.clone(), token_terminal.clone()))
                    {
                        Some(index) => *index,
                        None => {
                            return (
                                trace,
                                Err(ParseError::InvalidToken {
                                    token: token.clone(),
                                }),
                            )
                        }
                    };
                    let rule = parse_table.rules[index].clone();
                    assert_eq!(nonterminal, &rule.lhs);
                    trace.push(ParseTrace {
                        stack: stack.clone(),
                        input: input.iter().map(|v| v.0.clone()).collect(),
                        rule: Some(index),
                    });
                    stack.pop();
                    stack.extend(rule.rhs.into_iter().rev());
                }
                None => {
                    let index = match parse_table.table.get(&(nonterminal.clone(), Terminal::Eos)) {
                        Some(index) => *index,
                        None => return (trace, Err(ParseError::UnexpectedEos)),
                    };
                    let rule = parse_table.rules[index].clone();
                    assert_eq!(nonterminal, &rule.lhs);
                    trace.push(ParseTrace {
                        stack: stack.clone(),
                        input: input.iter().map(|v| v.0.clone()).collect(),
                        rule: Some(index),
                    });
                    stack.pop();
                    stack.extend(rule.rhs.into_iter().rev());
                }
            },
            None => {
                return (
                    trace,
                    Err(ParseError::ExtraToken {
                        token: input.pop().unwrap().0,
                    }),
                )
            }
        }
    }
    (trace, Ok(()))
}

#[test]
fn test_parse() {
    let source = indoc! {"
        a + b
    "}
    .to_string();
    let preprocessed = preprocess(source).unwrap();
    let tokens = lex(preprocessed).unwrap();
    let (trace, result) = parse(LL1ParseTable::default(), tokens);
    result.unwrap();
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParserSolver;

impl Solver for ParserSolver {
    fn id(&self) -> String {
        "parser".to_string()
    }

    fn title(&self) -> String {
        "语法分析器的构造".to_string()
    }

    fn description(&self) -> View {
        "输入符号串.".into_view()
    }

    fn default_input(&self) -> String {
        "foo + bar + 80".to_string()
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
        let tokens = match lex(preprocessed) {
            Ok(tokens) => tokens,
            Err(e) => {
                return view! {
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "词法分析" </p>
                        <pre class="text-red-500"> { e.to_string() } </pre>
                    </div>
                }
                .into_view()
            }
        };
        let table = LL1ParseTable::default();
        let (trace, result) = parse(table.clone(), tokens);
        let trace_view = view! {
            <table>
                <thead>
                    <tr>
                        <th> "步骤" </th>
                        <th> "分析栈" </th>
                        <th> "余留输入串" </th>
                        <th> "所用产生式" </th>
                    </tr>
                </thead>
                <tbody> {
                    trace.into_iter().zip(1..).map(|(t, i)| view! {
                        <tr>
                            <td> { i } </td>
                            <td> { t.stack.into_iter().map(|t| t.to_string()).join(" ") } </td>
                            <td> {
                                t.input.into_iter().rev().map(|t| {
                                    let terminal: Terminal = t.try_into().unwrap();
                                    terminal.to_string()
                                }).join(" ")
                            } </td>
                            <td> {
                                t.rule.map(|index| table.rules[index].to_string())
                            } </td>
                        </tr>
                    }).collect_vec()
                } </tbody>
            </table>
        };
        match result {
            Ok(_) => view! {
                <div class="mb-10">
                    <p class="font-bold mb-2"> "语法分析" </p>
                    { trace_view }
                </div>
            }
            .into_view(),
            Err(e @ ParseError::InvalidToken { .. }) => view! {
                <div class="mb-10">
                    <p class="font-bold mb-2"> "语法分析" </p>
                    <pre class="text-red-500"> { e.to_string() } </pre>
                </div>
            }
            .into_view(),
            Err(e) => view! {
                <div class="mb-10">
                    <p class="font-bold mb-2"> "语法分析" </p>
                    <pre class="text-red-500"> { e.to_string() } </pre>
                    { trace_view }
                </div>
            }
            .into_view(),
        }
    }
}
