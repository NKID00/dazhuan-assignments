use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use itertools::Itertools;
use leptos::*;
use pest::{
    error::Error as PestError,
    iterators::{Pair, Pairs},
    pratt_parser::{Assoc, Op, PrattParser},
    Parser,
};
use pest_derive::Parser;

use shiyanyi::{Solver, KaTeX};

#[derive(Parser)]
#[grammar = "propositional_formula.pest"]
struct PropositionalFormulaParser;

#[derive(Debug, Clone)]
enum Expr {
    Literal(bool),
    Proposition(String),
    Negation(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: Operator,
        rhs: Box<Expr>,
    },
}

impl Expr {
    fn parse(input: &str) -> Result<Self, Box<PestError<Rule>>> {
        Ok(Self::from_expr(Self::into_pairs(input)?))
    }

    fn into_pairs(input: &str) -> Result<Pairs<Rule>, Box<PestError<Rule>>> {
        Ok(PropositionalFormulaParser::parse(Rule::formula, input)?
            .next()
            .unwrap()
            .into_inner())
    }

    fn from_term(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::proposition => {
                match pair.as_str().to_ascii_lowercase().as_str() {
                    "t" | "true" => Expr::Literal(true),
                    "f" | "false" => Expr::Literal(false),
                    _ => Expr::Proposition(pair.as_str().to_string())
                }
            },
            Rule::negation => {
                Expr::Negation(Box::new(Self::from_term(pair.into_inner().next().unwrap())))
            }
            Rule::expr => Expr::from_expr(pair.into_inner()),
            _ => unreachable!(),
        }
    }

    fn from_binop(lhs: Expr, op: Pair<Rule>, rhs: Expr) -> Self {
        Expr::BinOp {
            lhs: Box::new(lhs),
            op: match op.as_rule() {
                Rule::conjunction => Operator::Conjunction,
                Rule::disjunction => Operator::Disjunction,
                Rule::implication => Operator::Implication,
                Rule::equivalence => Operator::Equivalence,
                _ => unreachable!(),
            },
            rhs: Box::new(rhs),
        }
    }

    fn from_expr<'i, P: Iterator<Item = Pair<'i, Rule>>>(tokens: P) -> Self {
        PrattParser::new()
            .op(Op::infix(Rule::equivalence, Assoc::Left))
            .op(Op::infix(Rule::implication, Assoc::Left))
            .op(Op::infix(Rule::disjunction, Assoc::Left))
            .op(Op::infix(Rule::conjunction, Assoc::Left))
            .map_primary(Expr::from_term)
            .map_infix(Expr::from_binop)
            .parse(tokens)
    }

    fn propositions(&self) -> HashSet<&str> {
        let mut propositions = HashSet::new();
        match self {
            Expr::Literal(_v) => {}
            Expr::Proposition(proposition) => {
                propositions.insert(proposition.as_str());
            }
            Expr::Negation(expr) => {
                propositions.extend(expr.propositions());
            }
            Expr::BinOp { lhs, op: _, rhs } => {
                propositions.extend(lhs.propositions());
                propositions.extend(rhs.propositions());
            }
        }
        propositions
    }

    fn substitute(&self, assignment: &Assignment) -> bool {
        match self {
            Expr::Literal(v) => *v,
            Expr::Proposition(proposition) => assignment[proposition.as_str()],
            Expr::Negation(expr) => !expr.substitute(assignment),
            Expr::BinOp { lhs, op, rhs } => {
                let (lhs, rhs) = (lhs.substitute(assignment), rhs.substitute(assignment));
                match op {
                    Operator::Conjunction => lhs && rhs,
                    Operator::Disjunction => lhs || rhs,
                    Operator::Implication => (!lhs) || rhs,
                    Operator::Equivalence => lhs == rhs,
                }
            }
        }
    }

    fn truth_table(&self) -> TruthTable {
        let propositions = self.propositions().into_iter().sorted().collect_vec();
        let possible_inputs = itertools::repeat_n([true, false].into_iter(), propositions.len())
            .multi_cartesian_product();
        possible_inputs
            .map(|inputs| {
                let assignment = propositions
                    .clone()
                    .into_iter()
                    .zip_eq(inputs)
                    .collect::<HashMap<_, _>>()
                    .into();
                let result = self.substitute(&assignment);
                (assignment, result)
            })
            .collect::<Vec<_>>()
            .into()
    }
}

#[derive(Debug, Clone)]
enum Operator {
    Conjunction,
    Disjunction,
    Implication,
    Equivalence,
}

struct Assignment<'a>(HashMap<&'a str, bool>);

impl<'a> From<HashMap<&'a str, bool>> for Assignment<'a> {
    fn from(value: HashMap<&'a str, bool>) -> Self {
        Self(value)
    }
}

impl<'a> Deref for Assignment<'a> {
    type Target = HashMap<&'a str, bool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct TruthTable<'a>(Vec<(Assignment<'a>, bool)>);

impl TruthTable<'_> {
    fn conjunctive_normal_form(&self) -> String {
        self.iter()
            .filter_map(|(assignment, result)| {
                if *result {
                    None
                } else {
                    Some(format!(
                        r" \left({}\right) ",
                        assignment
                            .keys()
                            .sorted()
                            .map(|p| {
                                if assignment[p] {
                                    format!(r"\lnot {}", p)
                                } else {
                                    format!("{}", p)
                                }
                            })
                            .join(r" \lor "),
                    ))
                }
            })
            .join(r" \land ")
    }

    fn disjunctive_normal_form(&self) -> String {
        self.iter()
            .filter_map(|(assignment, result)| {
                if *result {
                    Some(format!(
                        r" \left({}\right) ",
                        assignment
                            .keys()
                            .sorted()
                            .map(|p| {
                                if assignment[p] {
                                    format!("{}", p)
                                } else {
                                    format!(r" \lnot {}", p)
                                }
                            })
                            .join(r" \land "),
                    ))
                } else {
                    None
                }
            })
            .join(r" \lor ")
    }
}

impl<'a> From<Vec<(Assignment<'a>, bool)>> for TruthTable<'a> {
    fn from(value: Vec<(Assignment<'a>, bool)>) -> Self {
        Self(value)
    }
}

impl<'a> Deref for TruthTable<'a> {
    type Target = Vec<(Assignment<'a>, bool)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DiscreteMathematicsExp1 {
    title: String,
}

impl Solver for DiscreteMathematicsExp1 {
    fn title(&mut self) -> String {
        self.title.clone()
    }

    fn default_input(&mut self) -> String {
        "((P ∧ (T → Q)) → ¬(R ⇄ Q)) ∧ ¬S".to_string()
    }

    fn solve(&mut self, input: String) -> View {
        let expr = match Expr::parse(input.as_str()) {
            Ok(expr) => expr,
            Err(e) => {
                return view! {
                    <pre class="text-red-500"> {
                        format!("error: invalid syntax \n{}", e.with_path("<Input Section>"))
                    } </pre>
                }
                .into_view()
            }
        };
        let propositions = expr.propositions().into_iter().sorted().collect_vec();
        self.title = format!("DiscreteMathematicsExp1 (with {} propositions)", propositions.len());
        let truth_table = expr.truth_table();
        view! {
            <div class="mb-10">
                <p class="font-bold mb-2"> "Conjunctive normal form" </p>
                <KaTeX expr={ truth_table.conjunctive_normal_form() } />
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "Disjunctive normal form" </p>
                <KaTeX expr={ truth_table.disjunctive_normal_form() } />
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "Truth table" </p>
                <table class="truth-table">
                    <thead>
                        <tr>
                            {
                                propositions.iter().map(|p| view! {
                                    <th><KaTeX expr={ p.to_string() } /></th>
                                }).collect_vec()
                            }
                            <th><KaTeX expr={ input.clone() } /></th>
                        </tr>
                    </thead>
                    <tbody> {
                        truth_table.iter().map(|(assignment, result)| view! {
                            <tr>
                                {
                                    propositions.iter().map(|p| view! {
                                        <td><KaTeX expr={ if assignment[p] { r"\mathbf{T}" } else { r"\mathbf{F}" } } /></td>
                                    }).collect_vec()
                                }
                                <td><KaTeX expr={ if *result { r"\mathbf{T}" } else { r"\mathbf{F}" } } /></td>
                            </tr>
                        }).collect_vec()
                    } </tbody>
                </table>
            </div>
        }
        .into_view()
    }
}

impl Default for DiscreteMathematicsExp1 {
    fn default() -> Self {
        Self {
            title: "DiscreteMathematicsExp1".to_string(),
        }
    }
}

fn main() {
    DiscreteMathematicsExp1::boot();
}
