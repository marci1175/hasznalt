use client::Searchbar;
use yew::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
//     async fn invoke(cmd: &str, args: JsValue) -> JsValue;
// }

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>
        <div id="navigation">
            <button>{"Regisztráció"}</button>
            <button>{"Belépés"}</button>
        </div>

        <div id="main_search">
            <h1>{ "Hasznalt.hu" }</h1>
            <Searchbar default_text="Keresés"/>
        </div>
        </>
    }
}
