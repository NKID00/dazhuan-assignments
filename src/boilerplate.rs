use std::{marker::PhantomData, panic};

use base64::prelude::*;
use leptos::*;

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
    view! {
        <h1 class="text-4xl mt-16 mb-4 px-10"> { title } </h1>
        <div class="bg-white px-10 pt-10 pb-12 flex flex-col justify-start items-stretch gap-4 rounded-2xl border-2 shadow-xl">
            <p class="text-2xl font-bold mb-6"> "Input Section" </p>
            <textarea class="font-mono rounded h-[12rem] mx-8 p-2 border-2 border-sky-300" node_ref=input on:input=move |ev| {
                if let Some(location) = document().location() {
                    let _ = location.set_hash(BASE64_URL_SAFE_NO_PAD.encode(event_target_value(&ev).as_str()).as_str());
                }
            }> {
                previous_input
            } </textarea>
            <div class="flex justify-begin pt-4 px-8">
                <button class="rounded bg-sky-400 hover:bg-sky-500 active:bg-sky-600 text-white font-bold py-2 px-8" on:click=move |_| {
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
        </div>
        <div class="bg-white px-10 pt-10 pb-14 flex flex-col justify-start items-stretch gap-4 rounded-2xl border-2 shadow-xl">
            <p class="text-2xl font-bold mb-6"> {
                move || match duration() {
                    Some(duration) => format!("Answer Section (took {:.3}s)", duration),
                    None => "Answer Section".to_string()
                }
            } </p>
            <div class="min-h-[6rem] mx-8"> { move || answer().unwrap_or(().into_view()) } </div>
        </div>
        <div class="p-4" />
    }
}
