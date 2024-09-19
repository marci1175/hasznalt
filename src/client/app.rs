use yew::prelude::*;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
//     async fn invoke(cmd: &str, args: JsValue) -> JsValue;
// }

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main class="container">
            <h1>{ "Hello world" }</h1>
        </main>
    }
}
