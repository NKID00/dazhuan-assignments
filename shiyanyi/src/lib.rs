use std::{marker::PhantomData, panic};

use base64::prelude::*;
use leptos::*;
use leptos_meta::*;
use stylers::style_str;

#[macro_export]
macro_rules! println {
    ($($t:tt)*) => (leptos::logging::log!($($t)*))
}

// pub enum InputSpec {
//     Textarea {
//         description: View,
//         default_input: String,
//     },
// }

pub trait Solver
where
    Self: Default + Clone + PartialEq + 'static,
{
    fn title(&mut self) -> String;
    fn default_input(&mut self) -> String;
    fn solve(&mut self, input: String) -> View;
    fn boot() {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        mount_to_body(move || view! { <App _solver={ PhantomData::<Self> } /> });
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
fn App<S: Solver>(_solver: PhantomData<S>) -> impl IntoView {
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
    window_event_listener(ev::hashchange, move |_| {
        let new_input = parse_location_hash(default_input.as_str());
        if new_input != input().unwrap().value() {
            input().unwrap().set_value(new_input.as_str());
        }
    });
    let (answer, set_answer) = create_signal(None);
    let perf = window().performance();
    let (duration, set_duration) = create_signal(None::<f64>);
    let (class_name, style_val) = style_str! {
            .shiyanyi-solver {
                display: flex;
                margin: 0;
                font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
                flex-direction: column;
                justify-content: flex-start;
                align-items: stretch;
                width: 100%;
                min-width: 100%;
                height: 100%;
                min-height: 100%;
                gap: 3rem;
                padding-left: 10%;
                padding-right: 10%;
                padding-bottom: 4rem;
                background-color: rgb(241 245 249);
            }

            .shiyanyi-solver-title {
                padding-left: 2.5rem;
                padding-right: 2.5rem;
                margin-bottom: 1rem;
                margin-top: 4rem;
                font-size: 2.25rem;
                line-height: 2.5rem;
            }

            .shiyanyi-input-section {
                display: flex;
                padding: 2.5rem 2.5rem 3rem 2.5rem;
                flex-direction: column;
                gap: 1rem;
                justify-content: flex-start;
                align-items: stretch;
                border-radius: 1rem;
                border-width: 2px;
                background-color: #ffffff;
                box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1),
                    0 10px 10px -5px rgba(0, 0, 0, 0.04);
            }

            .shiyanyi-input-section > h2{
                margin-bottom: 1.5rem;
                font-size: 1.5rem;
                line-height: 2rem;
                font-weight: 700;
            }

            .shiyanyi-input-section > textarea {
                padding: 0.5rem;
                margin-left: 2rem;
                margin-right: 2rem;
                border-radius: 0.25rem;
                border-width: 2px;
                font-family: "DejaVu Sans Mono", ui-monospace, monospace;
                height: 12rem;
            }

            .shiyanyi-input-section > button {
                padding: 0.6rem 2.5rem;
                margin-left: 2rem;
                margin-right: 2rem;
                align-content: start;
                width: fit-content;
                border-radius: 0.25rem;
                font-weight: 700;
                color: #ffffff;
                background-color: rgb(56 189 248);
            }

            .shiyanyi-input-section > button:hover {
                background-color: rgb(14 165 233);
            }

            .shiyanyi-input-section > button:active {
                background-color: rgb(2 132 199);
            }

            .shiyanyi-answer-section {
                display: flex;
                padding: 2.5rem 2.5rem 3.5rem 2.5rem;
                flex-direction: column;
                gap: 1rem;
                justify-content: flex-start;
                align-items: stretch;
                border-radius: 1rem;
                border-width: 2px;
                background-color: #ffffff;
                box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1),
                    0 10px 10px -5px rgba(0, 0, 0, 0.04);
            }

            .shiyanyi-answer-section > h2 {
                margin-bottom: 1.5rem;
                font-size: 1.5rem;
                line-height: 2rem;
                font-weight: 700;
            }

            .shiyanyi-answer-section > div {
                margin-left: 2rem;
                margin-right: 2rem;
                margin-bottom: 1rem;
            }
        };
    view! {
        class = class_name,
        <Link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css" integrity="sha384-n8MVd4RsNIU0tAv4ct0nTaAbDJwPJzDEaqSD1odI+WdtXRGWt2kTvGFasHpSy3SV" crossorigin="anonymous" />
        <Script defer="" src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js" integrity="sha384-XjKyOOlGwcjNTAIQHIpgOno0Hl1YQqzUOEleOLALmuqehneUG+vnGctmUb0ZY0l8" crossorigin="anonymous" />
        <Script defer="" src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js" integrity="sha384-+VBxd3r6XgURycqtZ117nYw44OOcIax56Z4dCRWbxyPt0Koah1uHoK0o4+/RRE05" crossorigin="anonymous" />
        <Style>{style_val}</Style>
        <div class="shiyanyi-solver">
            <h1 class="shiyanyi-solver-title"> { title } </h1>
            <div class="shiyanyi-input-section">
                <h2> "Input Section" </h2>
                <textarea node_ref=input on:input=move |ev| {
                    if let Some(location) = document().location() {
                        let _ = location.set_hash(BASE64_URL_SAFE_NO_PAD.encode(event_target_value(&ev).as_str()).as_str());
                    }
                }> {
                    previous_input
                } </textarea>
                <button on:click=move |_| {
                    let begin = perf.as_ref().map(|p| p.now());
                    let mut solver_v = solver.get_untracked();
                    let answer = solver_v.solve(input().unwrap().value());
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
            <div class="shiyanyi-answer-section">
                <h2> {
                    move || match duration() {
                        Some(duration) => format!("Answer Section (took {:.3}s)", duration),
                        None => "Answer Section".to_string()
                    }
                } </h2>
                <div> { move || answer().unwrap_or(().into_view()) } </div>
            </div>
        </div>
    }
}

