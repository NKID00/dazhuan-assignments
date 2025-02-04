use std::{
    collections::{HashMap, VecDeque},
    fmt,
    io::Read,
    rc::Rc,
};

use base64::prelude::*;
use flate2::{
    bufread::{DeflateDecoder, DeflateEncoder},
    Compression,
};
use itertools::Itertools;
use js_sys::{Object, Reflect};
use leptos::*;
use leptos_dom::helpers::*;
use leptos_meta::*;
use leptos_router::*;
use stylers::style_str;
use wasm_bindgen::prelude::*;
use web_sys::HtmlScriptElement;

#[macro_export]
macro_rules! println {
    ($($t:tt)*) => (leptos::logging::log!($($t)*))
}

#[must_use]
#[derive(Debug)]
pub struct EmptyShiyanyiBuilder {
    base_path: String,
}

impl EmptyShiyanyiBuilder {
    pub fn base_path(self, base_path: impl ToString) -> Self {
        Self {
            base_path: base_path.to_string(),
        }
    }

    pub fn section(
        self,
        id: impl ToString,
        title: impl ToString,
        children: ShiyanyiBuilder,
    ) -> ShiyanyiBuilder {
        let builder = ShiyanyiBuilder {
            children: Vec::new(),
            base_path: self.base_path,
        };
        builder.section(id, title, children)
    }

    pub fn solver(self, solver: Box<dyn Solver>) -> ShiyanyiBuilder {
        let builder = ShiyanyiBuilder {
            children: Vec::new(),
            base_path: self.base_path,
        };
        builder.solver(solver)
    }

    pub fn solver_default<S>(self) -> ShiyanyiBuilder
    where
        S: Solver + Default + 'static,
    {
        self.solver(Box::new(S::default()))
    }
}

#[must_use]
#[derive(Debug)]
pub struct ShiyanyiBuilder {
    children: Vec<SectionOrSolver>,
    base_path: String,
}

impl ShiyanyiBuilder {
    pub fn base_path(self, base_path: impl ToString) -> Self {
        Self {
            base_path: base_path.to_string(),
            ..self
        }
    }

    pub fn section(mut self, id: impl ToString, title: impl ToString, children: Self) -> Self {
        let id = id.to_string();
        if id.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')) {
            panic!("id is not url safe: {}", id);
        }
        self.children.push(SectionOrSolver::Section {
            id,
            title: title.to_string(),
            children: children.children,
        });
        self
    }

    pub fn solver(mut self, solver: Box<dyn Solver>) -> Self {
        let id = solver.id();
        if id.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')) {
            panic!("id is not url safe: {}", id);
        }
        self.children.push(SectionOrSolver::Solver {
            id,
            toc_title: solver.toc_title(),
            solver: Rc::new(solver),
        });
        self
    }

    pub fn solver_default<S>(self) -> ShiyanyiBuilder
    where
        S: Solver + Default + 'static,
    {
        self.solver(Box::new(S::default()))
    }

    // TODO: pub fn alias(mut self, title: String, target: String) -> Self

    pub fn build(self) -> Shiyanyi {
        Shiyanyi {
            base_path: self.base_path,
            children: self.children,
        }
    }
}

#[derive(Debug)]
pub struct Shiyanyi {
    base_path: String,
    children: Vec<SectionOrSolver>,
}

impl Shiyanyi {
    pub fn builder() -> EmptyShiyanyiBuilder {
        EmptyShiyanyiBuilder {
            base_path: "".to_string(),
        }
    }

    pub fn boot(self, mount_point_element_id: &str) {
        let mount_point: web_sys::HtmlElement = document()
            .get_element_by_id(mount_point_element_id)
            .expect("cannot find mount point with specified id")
            .dyn_into()
            .unwrap();
        mount_point.replace_children_with_node_0();
        for attr in mount_point.get_attribute_names().into_iter() {
            let attr = attr.as_string().unwrap();
            if attr != "id" {
                mount_point.remove_attribute(attr.as_str()).unwrap();
            }
        }
        mount_to(
            mount_point,
            move || view! { <ShiyanyiComponent base_path={ self.base_path } solver_tree={ self.children } /> },
        );
    }
}

#[derive(Clone)]
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

impl fmt::Debug for SectionOrSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Section {
                id,
                title,
                children,
            } => f
                .debug_struct("Section")
                .field("id", id)
                .field("title", title)
                .field("children", children)
                .finish(),
            Self::Solver {
                id,
                toc_title,
                solver,
            } => f
                .debug_struct("Solver")
                .field("id", id)
                .field("toc_title", toc_title)
                .field("solver", &solver.title())
                .finish(),
        }
    }
}

type SolverObject = Rc<Box<dyn Solver>>;

/// All methods must be pure functional (return identical results for identical arguments).
pub trait Solver {
    fn id(&self) -> String;
    /// Title shown in table of contents (side bar).
    fn toc_title(&self) -> String {
        self.title()
    }
    /// Title shown in the main section.
    fn title(&self) -> String;
    fn description(&self) -> View;
    fn default_input(&self) -> String;
    fn solve(&self, input: String) -> View;
}

pub fn escape_uri_component(s: &str) -> String {
    js_sys::encode_uri_component(s).as_string().unwrap()
}

pub fn unescape_uri_component(s: &str) -> String {
    js_sys::decode_uri_component(s)
        .unwrap()
        .as_string()
        .unwrap()
}

fn set_location_hash_encoded(s: &str) {
    let mut deflate = Vec::new();
    DeflateEncoder::new(s.as_bytes(), Compression::best())
        .read_to_end(&mut deflate)
        .unwrap();
    let base64 = BASE64_URL_SAFE_NO_PAD.encode(deflate);
    document()
        .location()
        .unwrap()
        .set_hash(base64.as_str())
        .unwrap();
}

fn get_location_hash_decoded() -> Option<String> {
    location_hash()
        .and_then(|h| if h.is_empty() { None } else { Some(h) })
        .and_then(|h| {
            BASE64_URL_SAFE_NO_PAD
                .decode(h.splitn(2, '#').last().unwrap())
                .ok()
        })
        .and_then(|bytes| {
            let mut s = String::new();
            DeflateDecoder::new(bytes.as_slice())
                .read_to_string(&mut s)
                .ok()
                .map(|_| s)
        })
}

fn register_katex_load_callback(set_katex_loaded: WriteSignal<bool>, katex_src: &str) {
    let closure: Box<dyn FnMut(_)> = Box::new(move |_: web_sys::Event| {
        set_katex_loaded(true);
    });
    let closure = Closure::wrap(closure);
    let elements = document().get_elements_by_tag_name("script");
    let len = elements.length();
    let elements = (0..len).map(|i| elements.get_with_index(i));
    for element in elements {
        let Some(element) = element else {
            continue;
        };
        let Ok(element) = element.dyn_into::<HtmlScriptElement>() else {
            continue;
        };
        if element.src() == katex_src {
            element
                .add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())
                .unwrap();
            break;
        }
    }
    closure.forget();
}

#[component]
fn ShiyanyiComponent(base_path: String, solver_tree: Vec<SectionOrSolver>) -> impl IntoView {
    provide_meta_context();
    let (map_path_solver, set_map_path_solver) = create_signal(HashMap::new());
    let (katex_loaded, set_katex_loaded) = create_signal(false);
    let katex_src = "https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js";
    let element = create_node_ref();
    element.on_load(move |_| {
        register_katex_load_callback(set_katex_loaded, katex_src);
    });
    let (class_name, style_val) = style_str! {
        :deep(#shiyanyi) {
            flex: 1;
        }
        .root {
            display: flex;
            flex-direction: row;
            justify-content: flex-start;
            align-items: stretch;
            width: 100%;
            min-height: 100%;
            padding: 3rem 5% 1rem 5%;
            color: rgb(63, 63, 66);
        }
        nav {
            display: flex;
            flex-direction: column;
            justify-content: flex-start;
            align-items: stretch;
            align-self: start;
            margin: 4rem 1.5rem 0 0;
            padding: 1rem 0 1rem 1rem;
            border-radius: 0.5rem;
            background: rgb(255, 255, 255);
            box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
        }
        main {
            flex: 1;
            min-width: 0;
            display: flex;
            flex-direction: column;
            justify-content: stretch;
            align-items: stretch;
        }
        @media only screen and (max-width: 1024px) {
            .root {
                flex-direction: column;
                padding: 0 0 1rem 0;
            }
            nav {
                align-self: stretch;
                margin: 0;
                padding: 0;
                border-radius: 0;
                box-shadow: none;
                border-bottom: 2px solid rgb(229, 231, 235);
            }
            main {
                padding: 0 1rem 0 1rem;
            }
        }
    };
    view! {
        class = class_name,
        <Style> { style_val } </Style>
        <Link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css" integrity="sha384-nB0miv6/jRmo5UMMR1wu3Gz6NLsoTkbqJghGIsx//Rlm+ZU03BU6SQNC66uf4l5+" crossorigin="anonymous" />
        <Script defer="" src={ katex_src } integrity="sha384-7zkQWkzuo3B5mTepMUcHkMB5jZaolc2xDwL6VFqjFALcbeS9Ggm/Yr2r3Dy4lfFg" crossorigin="anonymous" />
        <Router>
            <div class="root" node_ref=element>
                <nav> <Contents base_path={ base_path.clone() } solver_tree set_map_path_solver /> </nav>
                <main>
                    <Routes base={ base_path }>
                        <Route path="" view=Outlet >
                            <Route path="*path" view=move || view! { <SolverWrapper map_path_solver katex_loaded /> } />
                        </Route>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn Contents(
    base_path: String,
    solver_tree: Vec<SectionOrSolver>,
    set_map_path_solver: WriteSignal<HashMap<String, SolverObject>>,
) -> impl IntoView {
    let path_selected = use_location().pathname;
    let path_selected =
        Signal::derive(move || with!(|path_selected| path_selected[1..].to_string()));
    // convert tree of solver into contents
    let mut stack_solver_tree = vec![VecDeque::from(solver_tree)];
    let mut stack_path = Vec::new();
    let mut stack_contents = vec![(String::new(), VecDeque::new())];
    let mut map_path_solver_value = HashMap::new();
    let mut default_path = None;
    let (class_name, style_val) = style_str! {
        details.header > summary {
            pointer-events: none;
            padding-left: 1rem;
            font-weight: 700;
            font-size: 1.25rem;
            list-style: none;
        }
        ol {
            display: flex;
            flex-direction: column;
            list-style: none;
        }
        ol.root {
            min-width: 12rem;
            max-width: 24rem;
        }
        ol.section {
            border-left: 1px solid rgb(205, 233, 255);
        }
        summary {
            padding: 0.7rem 1rem 0.7rem 0;
            font-weight: 700;
            cursor: pointer;
        }
        li.section {
            display: flex;
            flex-direction: column;
            margin-left: 1.5rem;
        }
        li.solver {
            padding: 0.7rem 1.5rem 0.7rem 1.5rem;
        }
        li.solver:hover {
            text-decoration: underline;
        }
        li.selected {
            font-weight: 700;
            background-color: rgb(205, 233, 255);
        }
        @media only screen and (max-width: 1024px) {
            ol.root {
                min-width: revert;
                max-width: revert;
            }
            details.header > summary {
                list-style: revert;
                pointer-events: revert;
            }
            li.section {
                margin-left: 2.5rem;
            }
            li.solver {
                padding: 0.7rem 2.5rem 0.7rem 2.5rem;
            }
        }
    };
    let contents = loop {
        match stack_solver_tree.pop() {
            Some(mut sub_solver_tree) => {
                match sub_solver_tree.pop_front() {
                    Some(SectionOrSolver::Section { id, title, children }) => {
                        stack_solver_tree.push(sub_solver_tree);
                        stack_solver_tree.push(VecDeque::from(children));
                        stack_path.push(id);
                        stack_contents.push((title, VecDeque::new()));
                    },
                    Some(SectionOrSolver::Solver { id, toc_title, solver }) => {
                        stack_solver_tree.push(sub_solver_tree);
                        match stack_contents.last_mut() {
                            Some(sub_contents) => {
                                let path = if stack_path.is_empty() {
                                    id
                                } else {
                                    format!("{}/{}", stack_path.iter().join("/"), id)
                                };
                                if map_path_solver_value.insert(path.clone(), solver).is_some() {
                                    panic!("paths of two solvers are the same: {}", path);
                                }
                                let path = if base_path.is_empty() {
                                    path
                                } else {
                                    format!("{}/{}", base_path, path)
                                };
                                if default_path.is_none() {
                                    default_path = Some(path.clone());
                                }
                                sub_contents.1.push_back(view! {
                                    class = class_name,
                                    <A href={ path.clone() }>
                                        <li class="solver" class:selected={
                                            move || with!(|path_selected| path_selected == &path)
                                        } > { toc_title } </li>
                                    </A>
                                }.into_view());
                            },
                            None => unreachable!(),
                        }
                    }
                    None /* a sub tree has been fully converted, pop it and sum up its views */ => {
                        match stack_contents.pop() {
                            Some(sub_contents) => {
                                let title = sub_contents.0;
                                let solvers = sub_contents.1.into_iter().collect_vec();
                                match stack_contents.last_mut() {
                                    Some(parent_sub_contents) => {
                                        stack_path.pop().unwrap();
                                        parent_sub_contents.1.push_back(view! {
                                            class = class_name,
                                            <li class="section">
                                                <details open="">
                                                    <summary> { title } </summary>
                                                    <ol class="section"> { solvers } </ol>
                                                </details>
                                            </li>
                                        }.into_view());
                                    },
                                    None /* parent is root, conversion finishes */ => {
                                        break solvers
                                    },
                                }
                            },
                            None => unreachable!(),
                        }
                    },
                }
            }
            None => unreachable!(),
        }
    };
    set_map_path_solver(map_path_solver_value);
    let default_path = default_path.unwrap();
    let navigate = use_navigate();
    create_effect(move |_| {
        if with!(
            |path_selected| path_selected.is_empty() || path_selected == &format!("{}/", base_path)
        ) {
            navigate(default_path.as_str(), Default::default());
        }
    });
    let header = create_node_ref();
    let mobile = window()
        .match_media("only screen and (max-width: 1024px)")
        .unwrap()
        .unwrap()
        .matches();
    view! {
        class = class_name,
        <Style> { style_val } </Style>
        <details class="header" open={ if mobile { None } else { Some("") } } _ref=header>
            <summary> "Contents" </summary>
            <ol class="root">
                { contents }
            </ol>
        </details>
    }
}

#[component]
fn SolverWrapper(
    map_path_solver: ReadSignal<HashMap<String, SolverObject>>,
    katex_loaded: ReadSignal<bool>,
) -> impl IntoView {
    let (class_name_not_found, style_val_not_found) = style_str! {
        div {
            flex: 1;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: stretch;
        }
        h1 {
            font-size: 3rem;
            text-align: center;
        }
    };
    let (class_name, style_val) = style_str! {
        .solver {
            display: flex;
            margin: 0;
            flex-direction: column;
            justify-content: flex-start;
            align-items: stretch;
            gap: 1.5rem;
        }
        .solver-title {
            padding-left: 2.5rem;
            padding-right: 2.5rem;
            font-size: 2.25rem;
            font-weight: 900;
            line-height: 2.5rem;
        }
        .section {
            display: flex;
            padding: 2.5rem 2.5rem 3rem 2.5rem;
            flex-direction: column;
            gap: 1rem;
            justify-content: flex-start;
            align-items: stretch;
            border-radius: 0.75rem;
            background-color: rgb(255, 255, 255);
            box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
        }
        .section > h2{
            margin-bottom: 1rem;
            font-size: 1.5rem;
            line-height: 2rem;
            font-weight: 700;
        }
        .description > div {
            margin-left: 2rem;
            margin-right: 2rem;
            overflow: auto;
        }
        .input > textarea {
            padding: 0.5rem;
            margin-left: 2rem;
            margin-right: 2rem;
            border-radius: 0.25rem;
            border: 2px solid rgb(229, 231, 235);
            font-family: "DejaVu Sans Mono", ui-monospace, "Cascadia Code", Menlo,
            "Source Code Pro", Consolas, monospace;
            min-height: 12rem;
        }
        .input > button {
            padding: 0.6rem 2.5rem;
            margin-left: 2rem;
            margin-right: 2rem;
            align-self: start;
            width: fit-content;
            border-radius: 0.25rem;
            font-size: 1.2rem;
            font-weight: 700;
            color: rgb(255, 255, 255);
            background-color: rgb(125, 196, 255);
        }
        .input > button:hover {
            background-color: rgb(72, 158, 229);
        }
        .input > button:active {
            background-color: rgb(112, 175, 229);
        }
        .answer {
            flex: 1;
        }
        .answer > div {
            margin-left: 2rem;
            margin-right: 2rem;
            overflow: auto visible;
            min-height: 6rem;
        }
        @media only screen and (max-width: 1024px) {
            .solver {
                gap: 1rem;
            }
            .solver-title {
                margin-top: 1.5rem;
                font-size: 1.5rem;
                line-height: 2rem;
            }
            .section {
                padding: 0.75rem 1rem 1rem 1rem;
                border-radius: 0.5rem;
                gap: 0.7rem;
            }
            .section > h2{
                margin-bottom: 0rem;
                margin-left: 0rem;
                margin-right: 0rem;
                font-size: 1rem;
                line-height: 1.5rem;
            }
            .input > textarea {
                margin-left: 0;
                margin-right: 0;
            }
            .input > button {
                padding: 0.5rem 0rem;
                margin-left: 0;
                margin-right: 0;
                align-self: stretch;
                width: auto;
                font-size: 1rem;
            }
            .answer > div {
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
    create_effect(move |first_run| {
        if !katex_loaded() {
            return true;
        }
        with!(|s| document().set_title(
            s.as_ref()
                .map_or("Not Found".to_string(), |s| s.title())
                .as_str()
        ));
        if let Some(input) = input.get_untracked() {
            if first_run.unwrap_or(true) {
                if let Some(input_from_hash) = get_location_hash_decoded() {
                    input.set_value(input_from_hash.as_str());
                } else {
                    default_input
                        .with_untracked(|default_input| input.set_value(default_input.as_str()));
                }
            } else {
                default_input
                    .with_untracked(|default_input| input.set_value(default_input.as_str()));
            }
            set_duration(None);
            set_answer(None);
            false
        } else {
            true
        }
    });
    window_event_listener(ev::hashchange, move |_| {
        if let Some(input) = input() {
            if let Some(input_from_hash) = get_location_hash_decoded() {
                if input.value() != input_from_hash.as_str() {
                    input.set_value(input_from_hash.as_str());
                }
            }
        }
    });
    view! {
        class = class_name,
        <Style> { style_val_not_found } </Style>
        <Style> { style_val } </Style>
        <Show
            when=move || with!(move |s| s.is_some())
            fallback=move || view! {
                class = class_name_not_found,
                <div> <h1> "Not Found" </h1> </div>
            }
        >
            <Show
                when=katex_loaded
                fallback=move || view! {
                    class = class_name_not_found,
                    <div> <h1> "Loading" </h1> </div>
                }
            >
                <div class="solver">
                    <h1 class="solver-title"> { move || with!(move |s| s.as_ref().unwrap().title()) } </h1>
                    <div class="section description">
                        <h2> "Description." </h2>
                        <div> { move || with!(move |s| s.as_ref().unwrap().description()) } </div>
                    </div>
                    <div class="section input">
                        <h2> "Input." </h2>
                        <textarea node_ref=input />
                        <button on:click=move |_| {
                            let input = match input.get_untracked() {
                                Some(input) => input,
                                None => return,
                            };
                            let input_string = match input.value().as_str() {
                                "" => {
                                    let default_input = default_input.get_untracked();
                                    input.set_value(default_input.as_str());
                                    default_input
                                }
                                s => s.to_string(),
                            };
                            set_location_hash_encoded(input_string.as_str());
                            let begin = window().performance().unwrap().now();
                            let answer = s.with_untracked(|s| s.as_ref().unwrap().solve(input_string));
                            set_duration(Some(1.max((window().performance().unwrap().now() - begin) as u64)));
                            set_answer(Some(answer));
                        }> "Submit" </button>
                    </div>
                    <Show when=move || with!(|answer| answer.is_some())>
                        <div class="section answer">
                            <h2> {
                                move || with!(|duration| match duration {
                                    Some(duration) => format!("Answer. (took {}ms)", duration),
                                    None => "Answer.".to_string()
                                })
                            } </h2>
                            <div> { answer } </div>
                        </div>
                    </Show>
                </div>
            </Show>
        </Show>
    }
}

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
