use std::{
    fmt::Display,
    ops::{AddAssign, Mul, MulAssign},
};

use indoc::*;
use itertools::Itertools;
use leptos::*;
use num::{
    traits::{NumAssignRef, NumRef},
    BigRational, One, Signed, Zero,
};
use shiyanyi::*;

use crate::common::Matrix;

fn big_rational_to_string(x: &BigRational) -> String {
    if x.is_integer() {
        x.to_string()
    } else {
        format!(
            r"{}\frac{{{}}}{{{}}}",
            if x.is_negative() { "-" } else { "" },
            x.numer().abs(),
            x.denom()
        )
    }
}

fn big_rational_to_string_with_sign(x: &BigRational) -> String {
    if x.is_integer() {
        format!(
            r"{}{}",
            if x.is_negative() { "" } else { "+" },
            x.to_string()
        )
    } else {
        format!(
            r"{}\frac{{{}}}{{{}}}",
            if x.is_negative() { "-" } else { "+" },
            x.numer().abs(),
            x.denom()
        )
    }
}

fn big_rational_to_string_with_paren(x: &BigRational) -> String {
    if x.is_integer() {
        format!(
            r"{}{}{}",
            if x.is_negative() { r"\left(" } else { "" },
            x.to_string(),
            if x.is_negative() { r"\right)" } else { "" },
        )
    } else {
        format!(
            r"{}\frac{{{}}}{{{}}}{}",
            if x.is_negative() { r"\left(-" } else { "" },
            x.numer().abs(),
            x.denom(),
            if x.is_negative() { r"\right)" } else { "" },
        )
    }
}

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
) -> Vec<(String, Matrix<BigRational>)>
where
    BigRational: NumRef + NumAssignRef + Display + Clone,
{
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
                    big_rational_to_string_with_paren(&mul_inv)
                ),
                matrix.clone(),
            ));
        }
        for i in 0..matrix.shape().0 {
            if i != target_row && !matrix[i][j].is_zero() {
                let factor = -matrix[i][j].clone();
                matrix.scale_add_row(target_row, &factor, i);
                steps.push((
                    if factor.abs().is_one() {
                        format!(
                            r"r_{{{}}} {} r_{{{}}}",
                            i + 1,
                            if factor.is_positive() { "+" } else { "-" },
                            target_row + 1
                        )
                    } else {
                        format!(
                            r"r_{{{}}} {} r_{{{}}}",
                            i + 1,
                            big_rational_to_string_with_sign(&factor),
                            target_row + 1
                        )
                    },
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

impl ReducedRowEchelonForm for Matrix<BigRational>
where
    BigRational: NumRef + NumAssignRef + Display + Clone,
{
    fn reduced_row_echelon_form(&self) -> Self {
        match reduced_row_echelon_form_with_steps(&self).pop() {
            Some((_, matrix)) => matrix,
            None => self.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ReducedRowEchelon;

impl Solver for ReducedRowEchelon {
    fn id(&self) -> String {
        "reduced-row-echelon".to_string()
    }

    fn title(&self) -> String {
        "行最简形矩阵".to_string()
    }

    fn description(&self) -> View {
        "输入元素为整数或分数的矩阵.".into_view()
    }

    fn default_input(&self) -> String {
        indoc! {"
            1  3  1    3
            1  1 -1  1/3
            3 11  5 35/3
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
                <KaTeX expr={ format!(r"\begin{{pmatrix}}{}\end{{pmatrix}} \text{{已是行最简形矩阵.}}", matrix.map(big_rational_to_string)) } />
            }.into_view()
        } else {
            view! {
                <KaTeX display_mode=true fleqn=true expr={
                    format!(r"\begin{{align*}} \begin{{pmatrix}}{}\end{{pmatrix}} \  {} \end{{align*}}", matrix.map(big_rational_to_string), steps.into_iter().map(|(step, result)| {
                        format!(r"& \begin{{CD}}\\@>{{{step}}}>>\\\end{{CD}} \begin{{pmatrix}}{}\end{{pmatrix}}", result.map(big_rational_to_string))
                    }).join(r" \\[3em] "))
                } />
            }.into_view()
        }
    }
}
