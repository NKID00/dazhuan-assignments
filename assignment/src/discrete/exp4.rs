use leptos::*;

use shiyanyi::{KaTeX, Solver};

#[derive(Debug, Clone, PartialEq)]
pub struct Exp4;

impl Solver for Exp4 {
    fn id(&self) -> String {
        "exp4".to_string()
    }

    fn title(&self) -> String {
        "图的随机生成及欧拉（回）路的确定".to_string()
    }

    fn description(&self) -> View {
        ().into_view()
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
