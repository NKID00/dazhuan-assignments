use std::{collections::HashMap, fmt::Display, sync::OnceLock};

use indoc::indoc;
use itertools::Itertools;
use leptos::*;
use leptos_meta::Style;
use shiyanyi::*;
use stylers::style_str;
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
            Terminal::Ident => write!(f, "\\textrm{{Ident}}"),
            Terminal::Sym(Sym::LeftParen) => write!(f, "\\texttt{{(}}"),
            Terminal::Sym(Sym::RightParen) => write!(f, "\\texttt{{)}}"),
            Terminal::Op(Op::Add) => write!(f, "\\texttt{{+}}"),
            Terminal::Op(Op::Mul) => write!(f, "\\texttt{{*}}"),
            Terminal::LiteralInt => write!(f, "\\textrm{{LiteralInt}}"),
            Terminal::Eos => write!(f, "\\#"),
            _ => panic!("invalid terminal symbol"),
        }
    }
}

impl TryFrom<Token> for Terminal {
    type Error = Token;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token {
            TokenValue::Ident(_) => Ok(Self::Ident),
            TokenValue::Sym(Sym::LeftParen) => Ok(Self::Sym(Sym::LeftParen)),
            TokenValue::Sym(Sym::RightParen) => Ok(Self::Sym(Sym::RightParen)),
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
            write!(f, "{}", self.rhs.iter().map(|t| t.to_string()).join("\\ "))
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
                                name: "E^\\prime".to_string(),
                            }),
                        ],
                    },
                    // E' ::= + T E'
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "E^\\prime".to_string(),
                        },
                        rhs: vec![
                            Term::Terminal(Terminal::Op(Op::Add)),
                            Term::Nonterminal(Nonterminal {
                                name: "T".to_string(),
                            }),
                            Term::Nonterminal(Nonterminal {
                                name: "E^\\prime".to_string(),
                            }),
                        ],
                    },
                    // E' ::= \epsilon
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "E^\\prime".to_string(),
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
                                name: "T^\\prime".to_string(),
                            }),
                        ],
                    },
                    // T' ::= * F T'
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "T^\\prime".to_string(),
                        },
                        rhs: vec![
                            Term::Terminal(Terminal::Op(Op::Mul)),
                            Term::Nonterminal(Nonterminal {
                                name: "F".to_string(),
                            }),
                            Term::Nonterminal(Nonterminal {
                                name: "T^\\prime".to_string(),
                            }),
                        ],
                    },
                    // T' ::= \epsilon
                    LL1Rule {
                        lhs: Nonterminal {
                            name: "T^\\prime".to_string(),
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
                            Terminal::Sym(Sym::LeftParen),
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
                                name: "E^\\prime".to_string(),
                            },
                            Terminal::Op(Op::Add),
                        ),
                        1,
                    ),
                    // E' ( 2
                    (
                        (
                            Nonterminal {
                                name: "E^\\prime".to_string(),
                            },
                            Terminal::Sym(Sym::RightParen),
                        ),
                        2,
                    ),
                    // E' # 2
                    (
                        (
                            Nonterminal {
                                name: "E^\\prime".to_string(),
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
                            Terminal::Sym(Sym::LeftParen),
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
                                name: "T^\\prime".to_string(),
                            },
                            Terminal::Op(Op::Add),
                        ),
                        5,
                    ),
                    // T' * 4
                    (
                        (
                            Nonterminal {
                                name: "T^\\prime".to_string(),
                            },
                            Terminal::Op(Op::Mul),
                        ),
                        4,
                    ),
                    // T' ) 5
                    (
                        (
                            Nonterminal {
                                name: "T^\\prime".to_string(),
                            },
                            Terminal::Sym(Sym::RightParen),
                        ),
                        5,
                    ),
                    // T' # 5
                    (
                        (
                            Nonterminal {
                                name: "T^\\prime".to_string(),
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
                            Terminal::Sym(Sym::LeftParen),
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
enum ParseTraceRowRule {
    Rule(usize),
    None,
    Err,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParseTraceRow {
    stack: Vec<Term>,
    input: Vec<Token>,
    rule: ParseTraceRowRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParseTrace(Vec<ParseTraceRow>);

impl From<Vec<ParseTraceRow>> for ParseTrace {
    fn from(value: Vec<ParseTraceRow>) -> Self {
        Self(value)
    }
}

impl ParseTrace {
    fn into_view_with_table(self, table: LL1ParseTable) -> View {
        let (class_name, style_val) = style_str! {
            thead > tr {
                border-top: 1px solid #333;
                border-bottom: 1px solid #333;
            }

            tbody > tr:last-child {
                border-bottom: 1px solid #333;
            }

            th:first-child, td:first-child {
                text-align: center;
                border-left: 1px solid #333;
            }

            th:nth-child(2), td:nth-child(2) {
                text-align: left;
                border-left: 1px solid #333;
            }

            th:nth-child(3), td:nth-child(3) {
                text-align: right;
            }

            th:last-child {
                text-align: center;
                border-left: 1px solid #333;
                border-right: 1px solid #333;
            }

            td:last-child {
                text-align: left;
                border-left: 1px solid #333;
                border-right: 1px solid #333;
            }

            th, td {
                padding: 0.3rem 1rem;
            }
        };
        view! {
            class = class_name,
            <Style> {style_val} </Style>
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
                    self.0.into_iter().zip(1..).map(|(t, i)| view! {
                        class = class_name,
                        <tr>
                            <td><KaTeX expr={ i.to_string() } /></td>
                            <td><KaTeX expr={
                                [Term::Terminal(Terminal::Eos)]
                                    .into_iter()
                                    .chain(t.stack.into_iter())
                                    .map(|t| t.to_string())
                                    .join("\\ ")
                            } /></td>
                            <td><KaTeX expr={
                                t.input
                                    .into_iter()
                                    .rev()
                                    .map(|t| format_token(t))
                                    .chain(["\\#".to_string()].into_iter())
                                    .join("\\ ")
                            } /></td>
                            <td> {
                                match t.rule {
                                    ParseTraceRowRule::Rule(index) => view! {
                                        class = class_name,
                                        <KaTeX expr={ table.rules[index].to_string() } />
                                    }.into_view(),
                                    ParseTraceRowRule::None => ().into_view(),
                                    ParseTraceRowRule::Err => view! {
                                        class = class_name,
                                        <pre class="text-red-500"> "Error" </pre>
                                    }.into_view(),
                                }
                            } </td>
                        </tr>
                    }).collect_vec()
                } </tbody>
            </table>
        }
        .into_view()
    }
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

fn parse(parse_table: LL1ParseTable, input: Vec<Token>) -> (ParseTrace, Result<(), ParseError>) {
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
        Err(token) => return (trace.into(), Err(ParseError::InvalidToken { token })),
    };
    while !(stack.is_empty() && input.is_empty()) {
        match stack.last() {
            Some(Term::Terminal(terminal)) => match input.last() {
                Some((token, token_terminal)) => {
                    if terminal == token_terminal {
                        trace.push(ParseTraceRow {
                            stack: stack.clone(),
                            input: input.iter().map(|v| v.0.clone()).collect(),
                            rule: ParseTraceRowRule::None,
                        });
                        stack.pop();
                        input.pop();
                    } else {
                        trace.push(ParseTraceRow {
                            stack: stack.clone(),
                            input: input.iter().map(|v| v.0.clone()).collect(),
                            rule: ParseTraceRowRule::Err,
                        });
                        return (
                            trace.into(),
                            Err(ParseError::UnexpectedToken {
                                token: token.clone(),
                            }),
                        );
                    }
                }
                None => {
                    trace.push(ParseTraceRow {
                        stack: stack.clone(),
                        input: input.iter().map(|v| v.0.clone()).collect(),
                        rule: ParseTraceRowRule::Err,
                    });
                    return (trace.into(), Err(ParseError::UnexpectedEos));
                }
            },
            Some(Term::Nonterminal(nonterminal)) => match input.last() {
                Some((token, token_terminal)) => {
                    let index = match parse_table
                        .table
                        .get(&(nonterminal.clone(), token_terminal.clone()))
                    {
                        Some(index) => *index,
                        None => {
                            trace.push(ParseTraceRow {
                                stack: stack.clone(),
                                input: input.iter().map(|v| v.0.clone()).collect(),
                                rule: ParseTraceRowRule::Err,
                            });
                            return (
                                trace.into(),
                                Err(ParseError::UnexpectedToken {
                                    token: token.clone(),
                                }),
                            );
                        }
                    };
                    let rule = parse_table.rules[index].clone();
                    assert_eq!(nonterminal, &rule.lhs);
                    trace.push(ParseTraceRow {
                        stack: stack.clone(),
                        input: input.iter().map(|v| v.0.clone()).collect(),
                        rule: ParseTraceRowRule::Rule(index),
                    });
                    stack.pop();
                    stack.extend(rule.rhs.into_iter().rev());
                }
                None => {
                    let index = match parse_table.table.get(&(nonterminal.clone(), Terminal::Eos)) {
                        Some(index) => *index,
                        None => {
                            trace.push(ParseTraceRow {
                                stack: stack.clone(),
                                input: input.iter().map(|v| v.0.clone()).collect(),
                                rule: ParseTraceRowRule::Err,
                            });
                            return (trace.into(), Err(ParseError::UnexpectedEos));
                        }
                    };
                    let rule = parse_table.rules[index].clone();
                    assert_eq!(nonterminal, &rule.lhs);
                    trace.push(ParseTraceRow {
                        stack: stack.clone(),
                        input: input.iter().map(|v| v.0.clone()).collect(),
                        rule: ParseTraceRowRule::Rule(index),
                    });
                    stack.pop();
                    stack.extend(rule.rhs.into_iter().rev());
                }
            },
            None => {
                trace.push(ParseTraceRow {
                    stack: stack.clone(),
                    input: input.iter().map(|v| v.0.clone()).collect(),
                    rule: ParseTraceRowRule::Err,
                });
                return (
                    trace.into(),
                    Err(ParseError::ExtraToken {
                        token: input.pop().unwrap().0,
                    }),
                );
            }
        }
    }
    trace.push(ParseTraceRow {
        stack: stack.clone(),
        input: input.iter().map(|v| v.0.clone()).collect(),
        rule: ParseTraceRowRule::None,
    });
    (trace.into(), Ok(()))
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

fn format_token(token: Token) -> String {
    match token.token {
        TokenValue::Ident(ident) => {
            format!("\\textrm{{Ident}}\\left(\\texttt{{{}}}\\right)", ident.name)
        }
        TokenValue::Sym(Sym::LeftParen) => "\\texttt{{(}}".to_string(),
        TokenValue::Sym(Sym::RightParen) => "\\texttt{{)}}".to_string(),
        TokenValue::Op(Op::Add) => "\\texttt{{+}}".to_string(),
        TokenValue::Op(Op::Mul) => "\\texttt{{*}}".to_string(),
        TokenValue::LiteralInt(literal_int) => {
            format!("\\textrm{{LiteralInt}}\\left({}\\right)", literal_int.value)
        }
        _ => panic!("invalid token as terminal symbol"),
    }
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
        "1 + 2 * foo + bar".to_string()
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
        match result {
            Ok(_) => view! {
                <div class="mb-10">
                    <p class="font-bold mb-2"> "语法分析" </p>
                    { trace.into_view_with_table(table) }
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
                    <pre class="text-red-500 mb-2"> { e.to_string() } </pre>
                    { trace.into_view_with_table(table) }
                </div>
            }
            .into_view(),
        }
    }
}
