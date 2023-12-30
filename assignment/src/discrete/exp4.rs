use leptos::*;

use shiyanyi::{KaTeX, Solver};

#[derive(Debug, Clone, PartialEq)]
pub struct Exp4;

impl Solver for Exp4 {
    fn title(&self) -> String {
        "图的随机生成及欧拉（回）路的确定".to_string()
    }

    fn default_input(&self) -> String {
        "".to_string()
    }

    fn solve(&self, input: String) -> View {
        ().into_view()
    }
}

impl Default for Exp4 {
    fn default() -> Self {
        Self
    }
}
