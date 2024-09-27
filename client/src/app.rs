use std::str::FromStr;

use client::{SearchButton, Searchbar};
use js_sys::Array;
use reqwest_wasm::{Method, Url};
use wasm_bindgen::JsValue;
use web_sys::{console, Request, RequestInit};
use yew::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
//     async fn invoke(cmd: &str, args: JsValue) -> JsValue;
// }

#[function_component(App)]
pub fn app() -> Html {
    let searchbar_text = use_state(|| String::from("Keresés"));
    let search_buffer = use_state(|| String::new());

    let on_click = Callback::from(|event| {
        wasm_bindgen_futures::spawn_local(async {
            console::debug_1(&JsValue::from_str(&format!("{:?}", reqwest_wasm::Request::new(Method::GET, Url::from_str("https://ujbudaiszechenyi.hu/api/lessontimes").unwrap()).body())));
        });
    });

    html! {
        <>
            <div id="navigation">
                <button>{"Regisztráció"}</button>
                <button>{"Belépés"}</button>
            </div>

            <div id="main_search">
            <h1>{ "Hasznalt.hu" }</h1>
                <div id="search_bar">
                    <Searchbar default_text={searchbar_text} text_buffer={search_buffer.clone()}/>
                    <SearchButton label={html!(<img src="public\\search.svg" height=20/>)} callback={on_click}/>
                </div>
            </div>
        </>
    }
}
