use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use indoc::*;
use itertools::{repeat_n, Itertools};
use leptos::*;
use num::{BigRational, One, Signed, Zero};
use shiyanyi::*;

use crate::common::*;

use super::Rank;

#[derive(Debug, Clone, PartialEq)]
pub struct LinearEquations(pub Matrix<BigRational>);

impl LinearEquations {
    pub fn is_homogeneous(&self) -> bool {
        let (_, n) = self.shape();
        self.iter().all(|r| r[n - 1].is_zero())
    }

    pub fn has_any_solution(&self) -> bool {
        Matrix::<BigRational>(
            self.iter()
                .map(|r| r[0..r.len() - 1].to_vec())
                .collect_vec(),
        )
        .rank()
            == self.rank()
    }

    pub fn has_infinite_solutions(&self) -> bool {
        self.rank() < self.shape().1 - 1
    }
}

impl Deref for LinearEquations {
    type Target = Matrix<BigRational>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LinearEquations {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for LinearEquations {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<Matrix<BigRational>>()?))
    }
}

impl Display for LinearEquations {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.is_empty() {
            let (_, n) = self.shape();
            write!(
                f,
                r"\left\{{\begin{{alignat*}}{{{}}} {} \end{{alignat*}}\right.",
                n + 1,
                self.iter()
                    .filter(|r| !r.iter().all(|v| v.is_zero()))
                    .map(|r| {
                        if r[0..(n - 1)].iter().all(|v| v.is_zero()) {
                            format!(
                                r"{} 0 & = \  & {} &",
                                repeat_n("& ", (n - 1) * 2).join(""),
                                r[n - 1].to_tex()
                            )
                        } else {
                            let mut terms = vec!["".to_string()];
                            let mut first_term = true;
                            for j in 0..n - 1 {
                                if r[j].is_zero() {
                                    terms.push("&".to_string());
                                    continue;
                                }
                                terms.push(format!(
                                    r" \  {} \  & {} x_{}",
                                    if first_term {
                                        first_term = false;
                                        r[j].sign_to_tex()
                                    } else {
                                        r[j].sign_to_tex_with_positive_sign()
                                    },
                                    r[j].abs().to_tex_ignore_one(),
                                    j + 1
                                ));
                            }
                            terms.push(format!(r" = \  & {} &", r[n - 1].to_tex()));
                            terms.join(" & ")
                        }
                    })
                    .join(" \\\\[1ex]\n")
            )?;
        }
        Ok(())
    }
}

pub trait Row {
    fn row(&self, row: usize) -> Vec<BigRational>;
}

impl Row for Matrix<BigRational> {
    fn row(&self, row: usize) -> Vec<BigRational> {
        self[row].clone()
    }
}

pub trait Col {
    fn col(&self, col: usize) -> Vec<BigRational>;
}

impl Col for Matrix<BigRational> {
    fn col(&self, col: usize) -> Vec<BigRational> {
        self.iter().map(|r| r[col].clone()).collect_vec()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LinearEquationsSolver;

impl Solver for LinearEquationsSolver {
    fn id(&self) -> String {
        "lineq".to_string()
    }

    fn title(&self) -> String {
        "实线性方程组".to_string()
    }

    fn description(&self) -> View {
        "输入元素为整数或分数的增广矩阵.".into_view()
    }

    fn default_input(&self) -> String {
        indoc! {"
            3 2 1 1 -3 0
            1 1 1 1  1 0
            0 1 2 2  6 0
            5 4 3 3 -1 0
        "}
        .to_string()
    }

    fn solve(&self, input: String) -> View {
        let matrix = match input.parse::<Matrix<BigRational>>() {
            Ok(matrix) => matrix,
            Err(_) => {
                return view! {
                    <p> "Failed to parse." </p>
                }
                .into_view()
            }
        };
        let (_, n) = matrix.shape();
        if n < 2 {
            return view! {
                <p> "Augmented matrix must contain at least 2 columns." </p>
            }
            .into_view();
        }
        let lineq = LinearEquations(matrix.clone());
        if lineq.is_homogeneous() {
            let reduced = LinearEquations(matrix.reduced_row_echelon_form());
            if reduced.has_infinite_solutions() {
                let mut main_unknowns = Vec::new();
                let mut basic_solutions = Vec::new();
                for j in 0..(n - 1) {
                    let mut col = reduced.col(j);
                    let nonzero = col
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| !x.is_zero())
                        .collect_vec();
                    if nonzero.is_empty() {
                        todo!(); // TODO: 无关未知量
                    } else if nonzero.len() == 1 {
                        let x = nonzero.last().unwrap().0;
                        if main_unknowns.contains(&x) {
                            for i in 0..col.len() {
                                col[i] = -&col[i];
                            }
                            col.extend(repeat_n(BigRational::zero(), n - 1 - col.len()));
                            col[j] = BigRational::one();
                            basic_solutions.push(col);
                        } else {
                            main_unknowns.push(x);
                        }
                    } else if nonzero.len() > 1 {
                        for i in 0..col.len() {
                            col[i] = -&col[i];
                        }
                        col.extend(repeat_n(BigRational::zero(), n - 1 - col.len()));
                        col[j] = BigRational::one();
                        basic_solutions.push(col);
                    }
                }
                let solution = format!(
                    r"\left\{{{} \mid {} \in \mathbb{{R}}\right\}}",
                    (0..basic_solutions.len())
                        .map(|i| format!(r"k_{} \bm\xi_{}", i + 1, i + 1))
                        .join(" + "),
                    (0..basic_solutions.len())
                        .map(|i| format!(r"k_{}", i + 1))
                        .join(" , ")
                );
                let basic_solutions = basic_solutions
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        format!(
                            r"\bm\xi_{} = \begin{{pmatrix}}{}\end{{pmatrix}}",
                            i + 1,
                            v.iter().map(BigRational::to_tex).join(r" \\[1ex] ")
                        )
                    })
                    .join(r",\ ");
                view! {
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "齐次实线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ lineq.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "增广矩阵的行最简形矩阵对应的齐次线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ reduced.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "方程组解的类型" </p>
                        <p> "有无穷多个解." </p>
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "一个基础解系" </p>
                        <KaTeX expr={ basic_solutions } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "方程组的解集" </p>
                        <KaTeX expr={ solution } />
                    </div>
                }
                .into_view()
            } else {
                view! {
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "齐次实线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ lineq.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "增广矩阵的行最简形矩阵对应的齐次线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ reduced.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "方程组解的类型" </p>
                        <p> "仅有零解." </p>
                    </div>
                }
                .into_view()
            }
        } else {
            let reduced = LinearEquations(matrix.reduced_row_echelon_form());
            if !reduced.has_any_solution() {
                view! {
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "非齐次实线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ lineq.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "增广矩阵的行最简形矩阵对应的非齐次线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ reduced.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "方程组解的类型" </p>
                        <p> "无解." </p>
                    </div>
                }
                .into_view()
            } else if reduced.has_infinite_solutions() {
                let mut main_unknowns = Vec::new();
                let mut basic_solutions = Vec::new();
                for j in 0..(n - 1) {
                    let mut col = reduced.col(j);
                    let nonzero = col
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| !x.is_zero())
                        .collect_vec();
                    if nonzero.is_empty() {
                        todo!(); // TODO: 无关未知量
                    } else if nonzero.len() == 1 {
                        let x = nonzero.last().unwrap().0;
                        if main_unknowns.contains(&x) {
                            for i in 0..col.len() {
                                col[i] = -&col[i];
                            }
                            col.extend(repeat_n(BigRational::zero(), n - 1 - col.len()));
                            col[j] = BigRational::one();
                            basic_solutions.push(col);
                        } else {
                            main_unknowns.push(x);
                        }
                    } else if nonzero.len() > 1 {
                        for i in 0..col.len() {
                            col[i] = -&col[i];
                        }
                        col.extend(repeat_n(BigRational::zero(), n - 1 - col.len()));
                        col[j] = BigRational::one();
                        basic_solutions.push(col);
                    }
                }
                let mut one_solution = basic_solutions.first().unwrap().clone();
                for i in 0..reduced.shape().0 {
                    one_solution[i] += &reduced[i][n - 1];
                }
                let one_solution = format!(
                    r"\bm\eta_0 = \begin{{pmatrix}}{}\end{{pmatrix}}",
                    one_solution
                        .iter()
                        .map(BigRational::to_tex)
                        .join(r" \\[1ex] ")
                );
                let solution = format!(
                    r"\left\{{\bm\eta_0 + {} \mid {} \in \mathbb{{R}}\right\}}",
                    (0..basic_solutions.len())
                        .map(|i| format!(r"k_{} \bm\xi_{}", i + 1, i + 1))
                        .join(" + "),
                    (0..basic_solutions.len())
                        .map(|i| format!(r"k_{}", i + 1))
                        .join(" , ")
                );
                let basic_solutions = basic_solutions
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        format!(
                            r"\bm\xi_{} = \begin{{pmatrix}}{}\end{{pmatrix}}",
                            i + 1,
                            v.iter().map(BigRational::to_tex).join(r" \\[1ex] ")
                        )
                    })
                    .join(r",\ ");
                view! {
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "非齐次实线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ lineq.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "增广矩阵的行最简形矩阵对应的非齐次线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ reduced.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "方程组解的类型" </p>
                        <p> "有无穷多个解." </p>
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "方程组的一个解" </p>
                        <KaTeX expr={ one_solution } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "对应的齐次线性方程组的一个基础解系" </p>
                        <KaTeX expr={ basic_solutions } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "方程组的解集" </p>
                        <KaTeX expr={ solution } />
                    </div>
                }
                .into_view()
            } else {
                view! {
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "非齐次实线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ lineq.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "增广矩阵的行最简形矩阵对应的非齐次线性方程组" </p>
                        <KaTeX display_mode=true fleqn=true expr={ reduced.to_string() } />
                    </div>
                    <div class="mb-10">
                        <p class="font-bold mb-2"> "方程组解的类型" </p>
                        <p> "有唯一解." </p>
                    </div>
                }
                .into_view()
            }
        }
    }
}
