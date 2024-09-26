use client::{SearchButton, Searchbar};
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
                    <SearchButton label={search_buffer}/>
                </div>
            </div>
        </>
    }
}
