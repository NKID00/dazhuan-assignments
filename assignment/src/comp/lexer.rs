use indoc::indoc;
use leptos::*;
use shiyanyi::*;

fn lex(source: String) -> () {
    todo!()
}

#[test]
fn test_lex() {
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LexerSolver;

impl Solver for LexerSolver {
    fn id(&self) -> String {
        "lexer".to_string()
    }

    fn title(&self) -> String {
        "词法分析器的构造".to_string()
    }

    fn description(&self) -> View {
        "输入类 C 语言源程序.".into_view()
    }

    fn default_input(&self) -> String {
        indoc!{"
            main()
            {
                int a, b;
                a = 10;
                b = a + 20;
            }
        "}.to_string()
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
