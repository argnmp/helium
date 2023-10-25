use leptos::leptos_dom::logging::console_log;
use wasm_bindgen::prelude::*;

fn greet() {
    console_log("hello there?") 
}

#[wasm_bindgen(start)]
pub fn start() {
    greet();
}

fn main() {
    println!("Hello, world!");
}
