use std::{
    fmt::{self, Display, Formatter},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use indoc::*;
use itertools::{repeat_n, Itertools};
use leptos::*;
use num::{BigRational, Zero};
use shiyanyi::*;

use crate::common::*;
use crate::linalg::LinearEquations;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vector(pub Vec<BigRational>);

impl Deref for Vector {
    type Target = Vec<BigRational>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.is_empty() {
            write!(
                f,
                r"\begin{{pmatrix}}{}\end{{pmatrix}}",
                self.iter().map(|x| x.to_string()).join(r" \\[1ex] ")
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorSet(pub Vec<Vector>);

impl VectorSet {
    pub fn to_matrix(&self) -> Matrix<BigRational> {
        let (m, n) = self.shape();
        let mut matrix = Matrix::<BigRational>(repeat_n(Vec::new(), m).collect_vec());
        for j in 0..n {
            for i in 0..m {
                matrix[i].push(self[j][i].clone());
            }
        }
        matrix
    }

    pub fn shape(&self) -> (usize, usize) {
        (self[0].len(), self.len())
    }

    pub fn is_in_span(&self, vector: &Vector) -> bool {
        let (m, _) = self.shape();
        if vector.len() != m {
            panic!("Vector is not the same size as VectorSet.");
        }
        let mut matrix = self.to_matrix();
        for i in 0..m {
            matrix[i].push(vector[i].clone());
        }
        LinearEquations(matrix).has_any_solution()
    }
}

impl Deref for VectorSet {
    type Target = Vec<Vector>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VectorSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for VectorSet {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let matrix = s.parse::<Matrix<BigRational>>()?;
        let (_, n) = matrix.shape();
        Ok(Self(
            (0..n)
                .map(|j| Vector(matrix.iter().map(|r| r[j].clone()).collect_vec()))
                .collect_vec(),
        ))
    }
}

impl Display for VectorSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.is_empty() {
            write!(
                f,
                "{}",
                self.iter().map(|vector| vector.to_string()).join(r",\  ")
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MaximalLinearlyIndependentSolver;

impl Solver for MaximalLinearlyIndependentSolver {
    fn id(&self) -> String {
        "maxlinind".to_string()
    }

    fn title(&self) -> String {
        "极大线性无关组".to_string()
    }

    fn description(&self) -> View {
        "输入元素为整数或分数的矩阵, 研究其列向量组.".into_view()
    }

    fn default_input(&self) -> String {
        indoc! {"
             1 0 0  3 0  1 2
            -1 3 3  0 0 -1 1
             2 1 1  7 0  2 5
             4 2 2 14 0  0 0
        "}
        .to_string()
    }

    fn solve(&self, input: String) -> View {
        let vector_set = match input.parse::<VectorSet>() {
            Ok(vector_set) => vector_set,
            Err(_) => {
                return view! {
                    <p> "Failed to parse." </p>
                }
                .into_view()
            }
        };
        let (_, n) = vector_set.shape();
        let mut unique_vector_set = VectorSet(Vec::new());
        for i in 0..n {
            if !unique_vector_set.contains(&vector_set[i]) {
                unique_vector_set.push(vector_set[i].clone());
            }
        }
        let unique_nonzero_vector_set = VectorSet(
            unique_vector_set
                .iter()
                .filter(|vector| !vector.iter().all(|x| x.is_zero()))
                .cloned()
                .collect_vec(),
        );
        let mut mutable_unique_nonzero_vector_set = unique_nonzero_vector_set.clone();
        let mut maximal_linearly_independent =
            VectorSet(vec![mutable_unique_nonzero_vector_set.remove(0)]);
        while !mutable_unique_nonzero_vector_set.is_empty() {
            let vector = mutable_unique_nonzero_vector_set.remove(0);
            if !maximal_linearly_independent.is_in_span(&vector) {
                maximal_linearly_independent.push(vector);
            }
        }
        view! {
            <div class="mb-10">
                <p class="font-bold mb-2"> "向量组" </p>
                <KaTeX expr={ vector_set.to_string() } />
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "一个极大线性无关组" </p>
                <KaTeX expr={ maximal_linearly_independent.to_string() } />
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "向量组的秩" </p>
                <KaTeX expr={
                    format!(
                        r"\mathrm{{r}}\left({}\right) = {}",
                        vector_set.to_string(),
                        maximal_linearly_independent.len()
                    )
                } />
            </div>
        }
        .into_view()
    }
}
