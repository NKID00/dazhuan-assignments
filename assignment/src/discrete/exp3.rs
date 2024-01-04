use std::collections::HashMap;

use indoc::*;
use itertools::Itertools;
use leptos::*;
use leptos_meta::Style;
use num::{BigInt, Zero};
use shiyanyi::*;
use stylers::style_str;

use crate::common::Matrix;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Exp3;

fn least_upper_bound(matrix: &Matrix<bool>, a: usize, b: usize) -> Option<usize> {
    let m = matrix.shape().0;
    let mut bound: Option<usize> = None;
    for i in 0..m {
        if matrix[a][i] && matrix[b][i] {
            match bound {
                Some(c) if matrix[i][c] => bound = Some(i),
                None => bound = Some(i),
                _ => {}
            }
        }
    }
    bound
}

fn greatest_lower_bound(matrix: &Matrix<bool>, a: usize, b: usize) -> Option<usize> {
    let m = matrix.shape().0;
    let mut bound: Option<usize> = None;
    for i in 0..m {
        if matrix[i][a] && matrix[i][b] {
            match bound {
                Some(c) if matrix[c][i] => bound = Some(i),
                None => bound = Some(i),
                _ => {}
            }
        }
    }
    bound
}

impl Solver for Exp3 {
    fn id(&self) -> String {
        "exp3".to_string()
    }

    fn title(&self) -> String {
        "偏序关系中盖住关系的求取及格论中有补格的判定".to_string()
    }

    fn description(&self) -> View {
        "输入集合元素和关系矩阵.".into_view()
    }

    fn default_input(&self) -> String {
        indoc! {"
            1 2 3 4 6 12

            1 1 1 1 1 1
            0 1 0 1 1 1
            0 0 1 0 1 1
            0 0 0 1 0 1
            0 0 0 0 1 1
            0 0 0 0 0 1
        "}
        .to_string()
    }

    fn solve(&self, input: String) -> View {
        let (set, matrix) = match input.split_once('\n') {
            Some(x) => x,
            None => return "Failed to parse.".into_view(),
        };
        let set = set.split_whitespace().collect_vec();
        let matrix = match matrix.parse::<Matrix<BigInt>>() {
            Ok(x) => x,
            Err(_) => return "Failed to parse.".into_view(),
        };
        let (m, n) = matrix.shape();
        if m != n {
            return "Matrix is not square.".into_view();
        }
        if m != set.len() {
            return "Incorrect element set.".into_view();
        }
        let matrix = matrix.map(|x| !x.is_zero());
        let mut covering /* 盖住关系 */ = Vec::new();
        for i in 0..m {
            for j in 0..m {
                if i != j
                    && matrix[i][j]
                    && !(0..m).any(|k| k != i && k != j && matrix[i][k] && matrix[k][j])
                {
                    covering.push((i, j));
                }
            }
        }
        let mut map_bound: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut is_lattice = true;
        for i in 0..m {
            for j in 0..m {
                let upper = least_upper_bound(&matrix, i, j);
                let lower = greatest_lower_bound(&matrix, i, j);
                if upper.is_none() || lower.is_none() {
                    is_lattice = false;
                    break;
                }
                map_bound.insert((i, j), (upper.unwrap(), lower.unwrap()));
            }
        }
        let complemented /* 有补格 */  = if is_lattice {
            let mut maximum  /* 最大元 */= None;
            for i in 0..m {
                if (0..m).all(|j| matrix[j][i]) {
                    maximum = Some(i)
                }
            }
            let maximum = maximum.unwrap();
            let mut minimum  /* 最小元 */= None;
            for i in 0..m {
                if (0..m).all(|j| matrix[i][j]) {
                    minimum = Some(i)
                }
            }
            let minimum = minimum.unwrap();
            (0..m).all(|i| (0..m).any(|j| map_bound[&(i, j)] == (maximum, minimum)))
        } else {
            false
        };
        let matrix = matrix.map(|x| if *x { "1" } else { "0" });
        let (class_name, style_val) = style_str! {
            tr {
                border-top: 1px solid #333;
                border-bottom: 1px solid #333;
            }
            th:first-child,
            td:first-child {
                border-left: 1px solid #333;
            }
            th:last-child,
            td:last-child {
                border-right: 1px solid #333;
            }
            th,
            td {
                text-align: center;
                padding: 0.3rem 1.5rem;
            }
        };
        view! {
                class = class_name,
                <Style> {style_val} </Style>
                <div class="mb-10">
                    <p class="font-bold mb-2"> "关系矩阵" </p>
                    <KaTeX expr={ format!(r"\begin{{bmatrix}} {} \end{{bmatrix}}", matrix) } />
                </div>
                <div class="mb-10">
                    <p class="font-bold mb-2"> "盖住关系" </p>
                    <p> { covering.iter().map(|(i, j)| format!("<{}, {}>", set[*i], set[*j])).join(", ") } </p>
                </div>
                <div class="mb-10">
                    <p class="font-bold mb-2"> "格的判定" </p>
                    <table>
                        <tbody>
                            <tr>
                                <td> "格" </td>
                                <td> { if is_lattice { "是" } else { "否" } } </td>
                            </tr>
                            <tr>
                                <td> "有补格" </td>
                                <td> { if complemented { "是" } else { "否" } } </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            }
            .into_view()
    }
}
