use leptos::*;

use shiyanyi::{KaTeX, Solver};

#[derive(Debug, Clone, PartialEq)]
pub struct Exp3;

impl Solver for Exp3 {
    fn title(&self) -> String {
        "偏序关系中盖住关系的求取及格论中有补格的判定".to_string()
    }

    fn default_input(&self) -> String {
        "".to_string()
    }

    fn solve(&self, input: String) -> View {
        ().into_view()
    }
}

impl Default for Exp3 {
    fn default() -> Self {
        Self
    }
}
