use client::{Button, Searchbar};
use wasm_bindgen::JsValue;
use web_sys::console;
use yew::prelude::*;
use yew_router::{BrowserRouter, Routable, Switch};

#[derive(Routable, Clone, Copy, PartialEq)]
enum Route {
    #[at("/")]
    Login,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Login => html! { <Login /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

#[function_component(Login)]
pub fn login_page() -> Html {
    let searchbar_text = use_state(|| String::from("Keresés"));
    let search_buffer = use_state(|| String::new());

    let on_click = Callback::from(|_| {
        wasm_bindgen_futures::spawn_local(async {
            console::debug_1(&JsValue::from_str(&reqwest::get("http://[::1]:3004/test").await.unwrap().text().await.unwrap()));
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
                    <Button label={html!(<img src="public\\search.svg" height=20/>)} callback={on_click}/>
                </div>
            </div>
        </>
    }
}
