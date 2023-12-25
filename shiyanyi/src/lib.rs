use std::{
    collections::{HashMap, VecDeque},
    panic,
    rc::Rc,
};

use base64::prelude::*;
use itertools::Itertools;
use js_sys::{Object, Reflect};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use stylers::style_str;
use wasm_bindgen::prelude::*;

#[macro_export]
macro_rules! println {
    ($($t:tt)*) => (leptos::logging::log!($($t)*))
}

#[must_use]
pub struct EmptyShiyanyiBuilder;

impl EmptyShiyanyiBuilder {
    pub fn section(self, id: String, title: String, children: ShiyanyiBuilder) -> ShiyanyiBuilder {
        let builder = ShiyanyiBuilder {
            children: Vec::new(),
        };
        builder.section(id, title, children)
    }

    pub fn solver(self, id: String, solver: Box<dyn Solver>) -> ShiyanyiBuilder {
        let builder = ShiyanyiBuilder {
            children: Vec::new(),
        };
        builder.solver(id, solver)
    }
}

#[must_use]
pub struct ShiyanyiBuilder {
    children: Vec<SectionOrSolver>,
}

impl ShiyanyiBuilder {
    pub fn section(mut self, id: String, title: String, children: Self) -> Self {
        self.children.push(SectionOrSolver::Section {
            id,
            title,
            children: children.children,
        });
        self
    }

    pub fn solver(mut self, id: impl ToString, solver: Box<dyn Solver>) -> Self {
        // FIXME: constrain id to ascii alphanumeric
        self.children.push(SectionOrSolver::Solver {
            id: id.to_string(),
            toc_title: solver.toc_title(),
            solver: Rc::new(solver),
        });
        self
    }

    pub fn build(self) -> Shiyanyi {
        Shiyanyi {
            children: self.children,
        }
    }
}

pub struct Shiyanyi {
    children: Vec<SectionOrSolver>,
}

impl Shiyanyi {
    pub fn builder() -> EmptyShiyanyiBuilder {
        EmptyShiyanyiBuilder
    }

    pub fn boot(self, path_prefix: String) {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        mount_to(
            crate::document().body().unwrap(),
            move || view! { <ShiyanyiComponent path_prefix solver_tree={ self.children } /> },
        );
    }
}

enum SectionOrSolver {
    Section {
        id: String,
        title: String,
        children: Vec<SectionOrSolver>,
    },
    Solver {
        id: String,
        toc_title: String,
        solver: SolverObject,
    },
}

type SolverObject = Rc<Box<dyn Solver>>;

/// All methods must be pure functional (return identical results for identical arguments).
pub trait Solver {
    /// Title shown in table of contents (side bar), will be calculated only once while booting.
    fn toc_title(&self) -> String {
        self.title()
    }
    /// Title shown in the main section.
    fn title(&self) -> String;
    fn default_input(&self) -> String;
    fn solve(&self, input: String) -> View;
}

fn parse_location_hash(default_input: &str) -> String {
    let hash_parsed =
        document()
            .location()
            .and_then(|l| l.hash().ok())
            .map_or("".to_string(), |h| {
                BASE64_URL_SAFE_NO_PAD
                    .decode(h.splitn(2, '#').last().unwrap())
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or("".to_string())
            });
    match hash_parsed.as_str() {
        "" => {
            if let Some(location) = document().location() {
                let _ = location.set_hash(BASE64_URL_SAFE_NO_PAD.encode(default_input).as_str());
            };
            default_input.to_string()
        }
        _ => hash_parsed,
    }
}

#[component]
fn ShiyanyiComponent(path_prefix: String, solver_tree: Vec<SectionOrSolver>) -> impl IntoView {
    let (class_name, style_val) = style_str! {
        .shiyanyi {
            display: flex;
            flex-direction: row;
            justify-content: flex-start;
            align-items: stretch;
        }
        .contents {
            display: flex;
            flex-direction: column;
            justify-content: flex-start;
            align-items: stretch;
        }
        .solver {
            flex: 1;
        }
        @media only screen and (max-width: 720px) {

        }
    };
    let (map_path_solver, set_map_path_solver) = create_signal(HashMap::new());
    view! {
        class = class_name,
        <Style> { style_val } </Style>
        <Router base={ Box::new(path_prefix.clone()).leak() }>
            <div class="shiyanyi">
                <nav class="contents"> <Contents solver_tree set_map_path_solver /> </nav>
                <main class="solver">
                    <Routes base={ path_prefix.clone() }>
                        <Route path="" view=Outlet >
                            <Route path="*path" view=move || view! { <SolverWrapper map_path_solver /> } />
                        </Route>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn Contents(
    solver_tree: Vec<SectionOrSolver>,
    set_map_path_solver: WriteSignal<HashMap<String, SolverObject>>,
) -> impl IntoView {
    let (class_name, style_val) = style_str! {
        .shiyanyi {

        }
        @media only screen and (max-width: 720px) {

        }
    };
    // convert tree of solver into contents
    let mut stack_solver_tree = vec![VecDeque::from(solver_tree)];
    let mut stack_path = Vec::new();
    let mut stack_contents = vec![(String::new(), VecDeque::new())];
    let mut map_path_solver_value = HashMap::new();
    let mut default_path = None;
    loop {
        match stack_solver_tree.pop() {
            Some(mut sub_solver_tree) => {
                match sub_solver_tree.pop_front() {
                    Some(SectionOrSolver::Section { id, title, children }) => {
                        stack_solver_tree.push(VecDeque::from(children));
                        stack_path.push(id);
                        stack_contents.push((title, VecDeque::new()));
                    },
                    Some(SectionOrSolver::Solver { id, toc_title, solver }) => {
                        stack_solver_tree.push(sub_solver_tree);
                        match stack_contents.last_mut() {
                            Some(sub_contents) => {
                                let path = format!("{}/{}", stack_path.iter().join("/"), id);
                                if default_path.is_none() {
                                    default_path = Some(path.clone());
                                }
                                if map_path_solver_value.insert(path.clone(), solver).is_some() {
                                    panic!("paths of two solvers are the same: {}", path);
                                }
                                sub_contents.1.push_back(view! {
                                    class = class_name,
                                    <li> <A href={ path }> { toc_title } </A> </li>
                                }.into_view());
                            },
                            None => unreachable!(),
                        }
                    }
                    None /* a sub tree has been fully converted, pop it and sum up its views */ => {
                        match stack_contents.pop() {
                            Some(sub_contents) => {
                                stack_path.pop().unwrap();
                                match stack_contents.last_mut() {
                                    Some(parent_sub_contents) => {
                                        let title = sub_contents.0;
                                        let solvers = sub_contents.1.into_iter().collect_vec();
                                        parent_sub_contents.1.push_back(view! {
                                            class = class_name,
                                            <p> { title } </p>
                                            <ul> { solvers } </ul>
                                        }.into_view());
                                    },
                                    None => unreachable!(),
                                }
                            },
                            None => unreachable!(),
                        }
                    },
                }
            }
            None => break,
        }
    }
    set_map_path_solver(map_path_solver_value);
    view! {
        class = class_name,
        <Style> { style_val } </Style>
        { stack_contents.pop().unwrap().1.into_iter().collect_vec() }
    }
}

#[component]
fn SolverWrapper(map_path_solver: ReadSignal<HashMap<String, SolverObject>>) -> impl IntoView {
    let (class_name, style_val) = style_str! {
        :deep(html, body) {
            width: 100%;
            height: 100%;
        }
        .solver {
            display: flex;
            margin: 0;
            font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
            flex-direction: column;
            justify-content: flex-start;
            align-items: stretch;
            width: 100%;
            min-height: 100%;
            gap: 2rem;
            padding-left: 10%;
            padding-right: 10%;
            padding-bottom: 4rem;
            background-color: rgb(238, 243, 249);
        }
        .solver-title {
            padding-left: 2.5rem;
            padding-right: 2.5rem;
            margin-top: 4rem;
            font-size: 2.25rem;
            line-height: 2.5rem;
        }
        .input-section {
            display: flex;
            padding: 2.5rem 2.5rem 3rem 2.5rem;
            flex-direction: column;
            gap: 1rem;
            justify-content: flex-start;
            align-items: stretch;
            border-radius: 1rem;
            border-width: 2px;
            background-color: rgb(255, 255, 255);
            box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1),
                0 10px 10px -5px rgba(0, 0, 0, 0.04);
        }
        .input-section > h2{
            margin-bottom: 1rem;
            font-size: 1.5rem;
            line-height: 2rem;
            font-weight: 700;
        }
        .input-section > textarea {
            padding: 0.5rem;
            margin-left: 2rem;
            margin-right: 2rem;
            border-radius: 0.25rem;
            border-width: 2px;
            font-family: "DejaVu Sans Mono", ui-monospace, monospace;
            height: 12rem;
        }
        .input-section > button {
            padding: 0.6rem 2.5rem;
            margin-left: 2rem;
            margin-right: 2rem;
            align-self: start;
            width: fit-content;
            border-radius: 0.25rem;
            font-weight: 700;
            color: rgb(255, 255, 255);
            background-color: rgb(125, 196, 255);
        }
        .input-section > button:hover {
            background-color: rgb(72, 158, 229);
        }
        .input-section > button:active {
            background-color: rgb(112, 175, 229);
        }
        .answer-section {
            flex: 1;
            display: flex;
            padding: 2.5rem 2.5rem 2rem 2.5rem;
            flex-direction: column;
            gap: 1rem;
            justify-content: flex-start;
            align-items: stretch;
            border-radius: 1rem;
            border-width: 2px;
            background-color: rgb(255, 255, 255);
            box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1),
                0 10px 10px -5px rgba(0, 0, 0, 0.04);
        }
        .answer-section > h2 {
            margin-bottom: 1rem;
            font-size: 1.5rem;
            line-height: 2rem;
            font-weight: 700;
        }
        .answer-section > div {
            margin-left: 2rem;
            margin-right: 2rem;
            overflow: scroll;
            min-height: 6rem;
        }
        @media only screen and (max-width: 720px) {
            .solver {
                padding-left: 1.5%;
                padding-right: 1.5%;
                gap: 1rem;
                padding-bottom: 1rem;
            }
            .solver-title {
                margin-top: 1.5rem;
                font-size: 1.5rem;
                line-height: 2rem;
            }
            .input-section {
                padding: 0.5rem 1rem 1rem 1rem;
                border-radius: 0.5rem;
                gap: 0.7rem;
            }
            .input-section > h2{
                margin-bottom: 0rem;
                margin-left: 0rem;
                margin-right: 0rem;
                font-size: 1rem;
                line-height: 1.5rem;
            }
            .input-section > textarea {
                margin-left: 0;
                margin-right: 0;
            }
            .input-section > button {
                padding: 0.5rem 0rem;
                margin-left: 0;
                margin-right: 0;
                align-self: stretch;
                width: auto;
            }
            .answer-section {
                padding: 0.5rem 1rem 1rem 1rem;
                border-radius: 0.5rem;
                gap: 0.7rem;
            }
            .answer-section > h2{
                margin-bottom: 0rem;
                margin-left: 0rem;
                margin-right: 0rem;
                font-size: 1rem;
                line-height: 1.5rem;
            }
            .answer-section > div {
                margin-left: 0;
                margin-right: 0;
            }
        }
    };
    let params = use_params_map();
    let path = Signal::derive(move || {
        with!(|params| params.get("path").unwrap_or(&"".to_string()).to_string())
    });
    let s =
        Signal::derive(move || with!(|path, map_path_solver| map_path_solver.get(path).cloned()));
    let input: NodeRef<html::Textarea> = create_node_ref();
    let default_input = Signal::derive(move || {
        with!(|s| s
            .as_ref()
            .map(|solver| solver.default_input())
            .unwrap_or_default())
    });
    let (answer, set_answer) = create_signal(None);
    let (duration, set_duration) = create_signal(None);
    create_effect(move |_| {
        with!(|s| document().set_title(
            s.as_ref()
                .map_or("Not Found".to_string(), |s| s.title())
                .as_str()
        ));
        if let Some(input) = input.get_untracked() {
            default_input.with_untracked(|default_input| input.set_value(default_input.as_str()));
        };
        set_duration(None);
        set_answer(None);
    });
    // TODO: include base64'd input in uri hash
    // window_event_listener(ev::hashchange, move |_| {
    //     if let Some(input) = input() {
    //         if let Some(default_input) =
    //             with!(move |solver| solver.clone().map(|solver| solver.default_input()))
    //         {
    //             let new_input = parse_location_hash(default_input.as_str());
    //             if new_input != input.value() {
    //                 input.set_value(new_input.as_str());
    //             }
    //         }
    //     }
    // });
    // let previous_input = parse_location_hash(default_input.as_str());
    // let previous_input = match previous_input.as_str() {
    //     "" => default_input.clone(),
    //     _ => previous_input,
    // };

    view! {
        class = class_name,
        <Link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css" integrity="sha384-n8MVd4RsNIU0tAv4ct0nTaAbDJwPJzDEaqSD1odI+WdtXRGWt2kTvGFasHpSy3SV" crossorigin="anonymous"></Link>
        <Script defer="" src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js" integrity="sha384-XjKyOOlGwcjNTAIQHIpgOno0Hl1YQqzUOEleOLALmuqehneUG+vnGctmUb0ZY0l8" crossorigin="anonymous"></Script>
        <Style> { style_val } </Style>
        <Show
            when=move || with!(move |s| s.is_some())
            fallback=|| view! { <p> "Not Found" </p> }
        >
            <div class="solver">
                <h1 class="solver-title"> { move || with!(move |s| s.as_ref().unwrap().title()) } </h1>
                <div class="input-section">
                    <h2> "Input Section" </h2>
                    <textarea node_ref=input />
                    <button on:click=move |_| {
                        let input = match input.get_untracked() {
                            Some(input) => input,
                            None => return,
                        };
                        let input_string = match input.value().as_str() {
                            "" => {
                                default_input.with_untracked(|default_input| {
                                    input.set_value(default_input);
                                    default_input.clone()
                                })
                            }
                            s => s.to_string(),
                        };
                        // if let Some(location) = document().location() {
                        //     let _ = location.set_hash(BASE64_URL_SAFE_NO_PAD.encode(input_string.as_str()).as_str());
                        // }
                        let begin = window().performance().unwrap().now();
                        let answer = s.with_untracked(|s| s.as_ref().unwrap().solve(input_string));
                        set_duration(Some(1.max((window().performance().unwrap().now() - begin) as u64)));
                        set_answer(Some(answer));
                    }> "Submit" </button>
                </div>
                <div class="answer-section">
                    <h2> {
                        move || with!(|duration| match duration {
                            Some(duration) => format!("Answer Section (took {}ms)", duration),
                            None => "Answer Section".to_string()
                        })
                    } </h2>
                    <div> { answer } </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn SingleSolverWrapper() -> impl IntoView {}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = katex, js_name = renderToString)]
    fn katex_render_to_string(expression: &str, options: &JsValue) -> String;
}

#[component]
pub fn KaTeX(
    #[prop(into)] expr: String,
    #[prop(default = false)] display_mode: bool,
    #[prop(default = false)] leqno: bool,
    #[prop(default = false)] fleqn: bool,
    #[prop(default = false)] throw_on_error: bool,
    #[prop(into, default = Object::new())] options: Object,
) -> impl IntoView {
    let options = Object::assign(&Object::new(), &options);
    Reflect::set(&options, &"displayMode".into(), &display_mode.into()).unwrap();
    Reflect::set(&options, &"leqno".into(), &leqno.into()).unwrap();
    Reflect::set(&options, &"fleqn".into(), &fleqn.into()).unwrap();
    Reflect::set(&options, &"throwOnError".into(), &throw_on_error.into()).unwrap();
    view! {
        <div inner_html={ katex_render_to_string(expr.as_str(), options.as_ref()) }></div>
    }
}
