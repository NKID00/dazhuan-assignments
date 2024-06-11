use leptos::*;
use shiyanyi::*;

fn parse(token_stream: Vec<String>) -> () {
    todo!()
}

#[test]
#[ignore = "not implemented"]
fn test_parser() {
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ParserSolver;

impl Solver for ParserSolver {
    fn id(&self) -> String {
        "parser".to_string()
    }

    fn title(&self) -> String {
        "语法分析器的构造".to_string()
    }

    fn description(&self) -> View {
        "输入符号串.".into_view()
    }

    fn default_input(&self) -> String {
        "TODO".to_string()
    }

    fn solve(&self, input: String) -> View {
        view! {
            <div class="mb-10">
                <p class="font-bold mb-2"> "TODO" </p>
            </div>
        }
        .into_view()
    }
}
