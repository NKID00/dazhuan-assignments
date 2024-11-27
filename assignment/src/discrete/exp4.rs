use itertools::{repeat_n, Itertools};
use leptos::*;
use leptos_meta::Style;
use num::Integer;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use shiyanyi::*;
use stylers::style_str;

use crate::common::Matrix;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Exp4;

fn dfs1(matrix: &Matrix<bool>, current: usize, visited: &mut Vec<bool>) {
    visited[current] = true;
    for i in 0..matrix.shape().0 {
        if !visited[i] && matrix[current][i] {
            dfs1(matrix, i, visited);
        }
    }
}

fn connected_component_count(matrix: &Matrix<bool>) -> usize {
    let mut visited = repeat_n(false, matrix.shape().0).collect_vec();
    let mut count = 0usize;
    while !visited.iter().all(|x| *x) {
        dfs1(
            matrix,
            visited
                .iter()
                .enumerate()
                .find(|(_, visited)| !*visited)
                .unwrap()
                .0,
            &mut visited,
        );
        count += 1;
    }
    count
}

impl Solver for Exp4 {
    fn id(&self) -> String {
        "exp4".to_string()
    }

    fn title(&self) -> String {
        "图的随机生成及欧拉（回）路的确定".to_string()
    }

    fn description(&self) -> View {
        "输入节点数, 边数和可选的随机种.".into_view()
    }

    fn default_input(&self) -> String {
        "10 20 1152921504606847241".to_string()
    }

    fn solve(&self, input: String) -> View {
        let mut input = input.split_whitespace();
        let vertex_count = match input.next().and_then(|s| s.parse::<usize>().ok()) {
            Some(v) => v,
            None => return "Failed to parse.".into_view(),
        };
        let edge_count = match input.next().and_then(|s| s.parse::<usize>().ok()) {
            Some(e) => e,
            None => return "Failed to parse.".into_view(),
        };
        let seed = match input.next().and_then(|s| s.parse::<u64>().ok()) {
            Some(s) => s,
            None => random(),
        };
        if edge_count > vertex_count * (vertex_count - 1) / 2 {
            return "Too many edges.".into_view();
        }
        let mut matrix = Matrix::<bool>(
            repeat_n(repeat_n(false, vertex_count).collect_vec(), vertex_count).collect_vec(),
        );
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
        let mut degree = repeat_n(0usize, vertex_count).collect_vec();
        for _ in 0..edge_count {
            loop {
                let a = rng.gen_range(0..vertex_count);
                let b = rng.gen_range(0..vertex_count);
                if a != b && !matrix[a][b] {
                    matrix[a][b] = true;
                    matrix[b][a] = true;
                    degree[a] += 1;
                    degree[b] += 1;
                    break;
                }
            }
        }
        let matrix = matrix;
        let is_connected = connected_component_count(&matrix) == 1;
        let (is_eulerian, is_semi_eulerian, path) = if is_connected {
            let mut odd_degree_vertices = Vec::new();
            for (i, d) in degree.iter().enumerate().take(vertex_count) {
                if d.is_odd() {
                    odd_degree_vertices.push(i)
                }
            }
            if odd_degree_vertices.len() > 2 {
                (false, false, Vec::new())
            } else {
                let mut matrix1 = matrix.clone();
                let mut path = Vec::new();
                let mut current = if odd_degree_vertices.is_empty() {
                    0usize
                } else {
                    odd_degree_vertices[0]
                };
                let mut previous_connected_component_count = connected_component_count(&matrix1);
                while matrix1[current].iter().any(|x| *x) {
                    path.push(current);
                    for next in 0..vertex_count {
                        if current != next && matrix1[current][next] {
                            matrix1[current][next] = false;
                            matrix1[next][current] = false;
                            let current_connected_component_count =
                                connected_component_count(&matrix1);
                            if current_connected_component_count
                                == previous_connected_component_count
                                || !matrix1[current].iter().any(|x| *x)
                            {
                                current = next;
                                previous_connected_component_count =
                                    current_connected_component_count;
                                break;
                            } else {
                                matrix1[current][next] = true;
                                matrix1[next][current] = true;
                            }
                        }
                    }
                }
                path.push(current);
                if odd_degree_vertices.is_empty() {
                    (true, false, path)
                } else {
                    (false, true, path)
                }
            }
        } else {
            (false, false, Vec::new())
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
                <p class="font-bold mb-2"> "随机种" </p>
                <p class="mb-6"> { seed } </p>
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "邻接矩阵" </p>
                <KaTeX expr={ format!(r"\begin{{bmatrix}} {} \end{{bmatrix}}", matrix) } />
            </div>
            <div class="mb-10">
                <p class="font-bold mb-2"> "图的判定" </p>
                <table>
                    <tbody>
                        <tr>
                            <td> "连通图" </td>
                            <td> { if is_connected { "是" } else { "否" } } </td>
                        </tr>
                        {
                            if is_connected {
                                view! {
                                    class = class_name,
                                    <tr>
                                        <td> "欧拉图" </td>
                                        <td> { if is_eulerian { "是" } else { "否" } } </td>
                                    </tr>
                                    <tr>
                                        <td> "半欧拉图" </td>
                                        <td> { if is_semi_eulerian { "是" } else { "否" } } </td>
                                    </tr>
                                }.into_view()
                            } else {
                                ().into_view()
                            }
                        }
                    </tbody>
                </table>
            </div>
            {
                if is_eulerian || is_semi_eulerian {
                    view! {
                        class = class_name,
                        <div class="mb-10">
                            <p class="font-bold mb-2"> { if is_eulerian { "欧拉回路" } else { "欧拉路" } } </p>
                            <p> { path.iter().map(|v| v.to_string()).join(" ") } </p>
                        </div>
                    }.into_view()
                } else {
                    ().into_view()
                }
            }
        }
        .into_view()
    }
}
