use std::{marker::PhantomData, panic};

use base64::prelude::*;
use js_sys::{Object, Reflect};
use leptos::*;
use leptos_meta::*;
use stylers::style_str;
use wasm_bindgen::prelude::*;

#[macro_export]
macro_rules! println {
    ($($t:tt)*) => (leptos::logging::log!($($t)*))
}

pub trait Solver
where
    Self: Default + Clone + PartialEq + 'static,
{
    fn title(&mut self) -> String;
    fn default_input(&mut self) -> String;
    fn solve(&mut self, input: String) -> View;
    fn boot() {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        mount_to_body(move || view! { <SolverComponent _solver={ PhantomData::<Self> } /> });
    }
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
fn SolverComponent<S: Solver>(_solver: PhantomData<S>) -> impl IntoView {
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
    let input: NodeRef<html::Textarea> = create_node_ref();
    let (solver, set_solver) = create_signal(<S as Default>::default());
    let title = create_memo(move |_| {
        let title = solver().title();
        document().set_title(title.as_str());
        title
    });
    let default_input = solver.get_untracked().default_input();
    let previous_input = parse_location_hash(default_input.as_str());
    let previous_input = match previous_input.as_str() {
        "" => default_input.clone(),
        _ => previous_input,
    };
    let default_input1 = default_input.clone();
    window_event_listener(ev::hashchange, move |_| {
        let new_input = parse_location_hash(default_input1.as_str());
        if new_input != input().unwrap().value() {
            input().unwrap().set_value(new_input.as_str());
        }
    });
    let (answer, set_answer) = create_signal(None);
    let perf = window().performance();
    let (duration, set_duration) = create_signal(None::<f64>);
    view! {
        class = class_name,
        <Link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css" integrity="sha384-n8MVd4RsNIU0tAv4ct0nTaAbDJwPJzDEaqSD1odI+WdtXRGWt2kTvGFasHpSy3SV" crossorigin="anonymous"></Link>
        <Script defer="" src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js" integrity="sha384-XjKyOOlGwcjNTAIQHIpgOno0Hl1YQqzUOEleOLALmuqehneUG+vnGctmUb0ZY0l8" crossorigin="anonymous"></Script>
        <Style>{style_val}</Style>
        <div class="solver">
            <h1 class="solver-title"> { title } </h1>
            <div class="input-section">
                <h2> "Input Section" </h2>
                <textarea node_ref=input> {
                    previous_input
                } </textarea>
                <button on:click=move |_| {
                    let input_string = input().unwrap().value();
                    let input_string = match input_string.as_str() {
                        "" => {
                            input().unwrap().set_value(default_input.as_str());
                            default_input.clone()
                        }
                        _ => input_string,
                    };
                    if let Some(location) = document().location() {
                        let _ = location.set_hash(BASE64_URL_SAFE_NO_PAD.encode(input_string.as_str()).as_str());
                    }
                    let begin = perf.as_ref().map(|p| p.now());
                    let mut solver_v = solver.get_untracked();
                    let answer = solver_v.solve(input_string);
                    if solver_v != solver.get_untracked() {
                        set_solver(solver_v);
                    }
                    match begin {
                        Some(begin) => set_duration(Some(0.001f64.max((perf.as_ref().unwrap().now() - begin) / 1000.0))),
                        None => set_duration(None),
                    }
                    set_answer(Some(answer));
                }> "Submit" </button>
            </div>
            <div class="answer-section">
                <h2> {
                    move || match duration() {
                        Some(duration) => format!("Answer Section (took {:.3}s)", duration),
                        None => "Answer Section".to_string()
                    }
                } </h2>
                <div> { answer } </div>
            </div>
        </div>
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
        <div inner_html={katex_render_to_string(expr.as_str(), options.as_ref())}></div>
    }
}
