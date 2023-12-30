use indoc::*;
use itertools::Itertools;
use leptos::*;

use leptos_meta::Style;
use num::{BigInt, Zero};
use shiyanyi::{KaTeX, Solver};
use stylers::style_str;

use crate::common::Matrix;

#[derive(Debug, Clone, PartialEq)]
pub struct Exp2;

impl Solver for Exp2 {
    fn id(&self) -> String {
        "exp2".to_string()
    }

    fn title(&self) -> String {
        "集合上二元关系性质判定".to_string()
    }

    fn description(&self) -> View {
        view! {
            <p> "输入关系矩阵." </p>
        }.into_view()
    }

    fn default_input(&self) -> String {
        indoc! {"
            1 1 1 0 0
            0 1 0 0 0
            0 0 1 1 0
            0 0 0 1 0
            0 0 0 0 1
        "}
        .to_string()
    }

    fn solve(&self, input: String) -> View {
        let matrix = match input.parse::<Matrix<BigInt>>() {
            Ok(matrix) => matrix,
            Err(_) => {
                return view! {
                    <p> "Failed to parse." </p>
                }
                .into_view()
            }
        };
        let (m, n) = matrix.shape();
        if m != n {
            return view! {
                <p> "Matrix is not square." </p>
            }
            .into_view();
        }
        let matrix = matrix.map(|x| !x.is_zero());
        let reflexive /* 自反性 */ = (0..m).all(|i| matrix[i][i]);
        let irreflexive /* 反自反性 */ = (0..m).all(|i| !matrix[i][i]);
        let symmetric /* 对称性 */ = (0..m).flat_map(|i| (0..i).map(|j| (i, j)).collect_vec()).all(|(i, j)| matrix[i][j] == matrix[j][i]);
        let antisymmetric /* 反对称性 */ = (0..m).flat_map(|i| (0..i).map(|j| (i, j)).collect_vec()).all(|(i, j)| !matrix[i][j] || !matrix[j][i]);
        // Warshall 算法
        let mut t /* 传递闭包 */ = matrix.clone();
        for i in 0..m {
            for j in 0..m {
                if t[j][i] {
                    for k in 0..m {
                        if t[i][k] {
                            t[j][k] = true;
                        }
                    }
                }
            }
        }
        let transitive /* 传递性 */ = matrix == t;
        let matrix = matrix.map(|x| if *x { "1" } else { "0" });
        let t = t.map(|x| if *x { "1" } else { "0" });
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
                <p class="font-bold mb-2"> "传递闭包的关系矩阵" </p>
                <KaTeX expr={ format!(r"\begin{{bmatrix}} {} \end{{bmatrix}}", t) } />
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "关系性质" </p>
                <table>
                    <tbody>
                        <tr>
                            <td> "自反性" </td>
                            <td> { if reflexive { "是" } else { "否" } } </td>
                        </tr>
                        <tr>
                            <td> "反自反性" </td>
                            <td> { if irreflexive { "是" } else { "否" } } </td>
                        </tr>
                        <tr>
                            <td> "对称性" </td>
                            <td> { if symmetric { "是" } else { "否" } } </td>
                        </tr>
                        <tr>
                            <td> "反对称性" </td>
                            <td> { if antisymmetric { "是" } else { "否" } } </td>
                        </tr>
                        <tr>
                            <td> "传递性" </td>
                            <td> { if transitive { "是" } else { "否" } } </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        }
        .into_view()
    }
}

impl Default for Exp2 {
    fn default() -> Self {
        Self
    }
}
