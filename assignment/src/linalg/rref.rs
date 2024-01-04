use std::ops::{AddAssign, Mul, MulAssign};

use indoc::*;
use itertools::Itertools;
use leptos::*;
use num::{BigRational, One, Zero};
use shiyanyi::*;

use crate::common::*;

pub trait SwapRow {
    /// row1 <-> row2
    fn swap_row(&mut self, row1: usize, row2: usize);
}

impl SwapRow for Matrix<BigRational> {
    fn swap_row(&mut self, row1: usize, row2: usize) {
        self.swap(row1, row2);
    }
}

pub trait ScaleRow {
    /// row *= factor
    fn scale_row<'a, U>(&mut self, row: usize, factor: &'a U)
    where
        BigRational: MulAssign<&'a U>;
}

impl ScaleRow for Matrix<BigRational> {
    fn scale_row<'a, U>(&mut self, row: usize, factor: &'a U)
    where
        BigRational: MulAssign<&'a U>,
    {
        for j in 0..self.shape().1 {
            self[row][j] *= factor;
        }
    }
}

pub trait ScaleAddRow {
    /// row2 += row1 * factor
    fn scale_add_row<'a, U>(&mut self, row1: usize, factor: &'a U, row2: usize)
    where
        BigRational: Clone,
        BigRational: Mul<&'a U>,
        BigRational: AddAssign<<BigRational as std::ops::Mul<&'a U>>::Output>;
}

impl ScaleAddRow for Matrix<BigRational> {
    fn scale_add_row<'a, U>(&mut self, row1: usize, factor: &'a U, row2: usize)
    where
        BigRational: Clone,
        BigRational: Mul<&'a U>,
        BigRational: AddAssign<<BigRational as std::ops::Mul<&'a U>>::Output>,
    {
        for j in 0..self.shape().1 {
            let x = (self[row1][j]).clone();
            self[row2][j] += x * factor;
        }
    }
}

fn reduced_row_echelon_form_with_steps(
    matrix: &Matrix<BigRational>,
) -> Vec<(String, Matrix<BigRational>)> {
    let mut matrix = matrix.clone();
    let mut steps = Vec::new();
    let mut target_row = 0;
    for j in 0..matrix.shape().1 {
        let mut first_non_zero_row = None;
        for i in target_row..matrix.shape().0 {
            if !matrix[i][j].is_zero() {
                first_non_zero_row = Some(i);
                break;
            }
        }
        let first_non_zero_row = match first_non_zero_row {
            Some(i) => i,
            None => continue,
        };
        if target_row != first_non_zero_row {
            matrix.swap_row(target_row, first_non_zero_row);
            steps.push((
                format!(
                    r"r_{{{}}} \leftrightarrow r_{{{first_non_zero_row}}}",
                    target_row + 1
                ),
                matrix.clone(),
            ));
        }
        if !matrix[target_row][j].is_one() {
            let mul_inv = BigRational::one() / &matrix[target_row][j];
            matrix.scale_row(target_row, &mul_inv);
            steps.push((
                format!(
                    r"r_{{{}}} \times {}",
                    target_row + 1,
                    mul_inv.to_tex_with_paren()
                ),
                matrix.clone(),
            ));
        }
        for i in 0..matrix.shape().0 {
            if i != target_row && !matrix[i][j].is_zero() {
                let factor = -matrix[i][j].clone();
                matrix.scale_add_row(target_row, &factor, i);
                steps.push((
                    format!(
                        r"r_{{{}}} {} r_{{{}}}",
                        i + 1,
                        factor.to_tex_with_sign_ignore_one(),
                        target_row + 1
                    ),
                    matrix.clone(),
                ));
            }
        }
        target_row += 1;
    }
    steps
}

pub trait ReducedRowEchelonForm {
    fn reduced_row_echelon_form(&self) -> Self;
}

impl ReducedRowEchelonForm for Matrix<BigRational> {
    fn reduced_row_echelon_form(&self) -> Self {
        match reduced_row_echelon_form_with_steps(&self).pop() {
            Some((_, matrix)) => matrix,
            None => self.clone(),
        }
    }
}

pub trait Rank {
    fn rank(&self) -> usize;
}

impl Rank for Matrix<BigRational> {
    fn rank(&self) -> usize {
        self.reduced_row_echelon_form()
            .iter()
            .filter(|r| !r.iter().all(|v| v.is_zero()))
            .count()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ReducedRowEchelonFormSolver;

impl Solver for ReducedRowEchelonFormSolver {
    fn id(&self) -> String {
        "rref".to_string()
    }

    fn title(&self) -> String {
        "行最简形矩阵".to_string()
    }

    fn description(&self) -> View {
        "输入元素为整数或分数的矩阵.".into_view()
    }

    fn default_input(&self) -> String {
        indoc! {"
            1   3  -2   5
            3   5   6   7
            1/2 1 1/2 3/2
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
        let steps = reduced_row_echelon_form_with_steps(&matrix);
        if steps.is_empty() {
            view! {
                <KaTeX expr={ format!(r"\begin{{pmatrix}}{}\end{{pmatrix}} \text{{已是行最简形矩阵.}}", matrix.map(BigRational::to_tex)) } />
            }.into_view()
        } else {
            let rref = steps.last().unwrap().1.clone();
            let rank = rref.rank();
            let rref = rref.to_tex();
            let matrix = matrix.to_tex();
            let steps = format!(
                r"\begin{{align*}} \begin{{pmatrix}}{}\end{{pmatrix}} {} \end{{align*}}",
                matrix,
                steps
                    .into_iter()
                    .map(|(step, result)| {
                        format!(
                            r"{}{step}{}{}{}",
                            r"& \begin{CD}\\@>{",
                            r"}>>\\\end{CD} \begin{pmatrix}",
                            result.map(BigRational::to_tex),
                            r"\end{pmatrix}"
                        )
                    })
                    .join(r" \\[3em] ")
            );
            view! {
                <div class="mb-10">
                    <p class="font-bold mb-2"> "行最简形矩阵" </p>
                    <KaTeX expr={ format!(r"\begin{{pmatrix}}{}\end{{pmatrix}}", rref) } />
                </div>
                <div class="mb-10">
                    <p class="font-bold mb-2"> "矩阵的秩" </p>
                    <KaTeX expr={ format!(r"\mathrm{{r}}\begin{{pmatrix}}{}\end{{pmatrix}} = {}", matrix, rank) } />
                </div>
                <div class="mb-10">
                    <p class="font-bold mb-2"> "初等行变换过程" </p>
                    <KaTeX display_mode=true fleqn=true expr={ steps } />
                </div>
            }
            .into_view()
        }
    }
}
