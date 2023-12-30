use leptos::*;

use shiyanyi::{KaTeX, Solver};

#[derive(Debug, Clone, PartialEq)]
pub struct Exp2;

impl Solver for Exp2 {
    fn title(&self) -> String {
        "集合上二元关系性质判定".to_string()
    }

    fn default_input(&self) -> String {
        "".to_string()
    }

    fn solve(&self, input: String) -> View {
        ().into_view()
    }
}

impl Default for Exp2 {
    fn default() -> Self {
        Self
    }
}
