use frontend::{Button, TextField};
use reqwest::Client;
use wasm_bindgen::JsValue;
use web_sys::{console, Request};
use yew::prelude::*;
use yew_router::{hooks::use_navigator, BrowserRouter, Routable, Switch};

#[derive(Routable, Clone, Copy, PartialEq)]
enum Route {
    #[at("/")]
    MainPage,
    #[at("/register")]
    Register,
    #[at("/login")]
    Login,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::MainPage => html! { <Main /> },
        Route::Register => html! { <Register /> },
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

#[function_component(Main)]
pub fn main_page() -> Html {
    let navigator = use_navigator().unwrap();
    let searchbar_text = use_state(|| String::from("Keresés"));
    let search_buffer = use_state(|| String::new());

    html! {
        <>
            <div id="navigation">
                <Button label={
                    "Regisztráció"
                } callback={
                    let navigator = navigator.clone();
                    Callback::from(move |_| {
                        navigator.push(&Route::Register);
                    })
                }/>

                <Button label={
                    "Bejelentkezés"
                } callback={
                    let navigator = navigator.clone();
                    Callback::from(move |_| {
                        navigator.push(&Route::Login);
                    })
                }/>
            </div>

            <div id="main_search">
            <h1>{ "Hasznalt.hu" }</h1>
                <div id="search_bar">
                    <TextField default_text={searchbar_text} text_buffer={search_buffer.clone()}/>
                    <Button id="search_button" label={html!(<img src="public\\search.svg" height=20/>)} callback={Callback::from(|_| {})}/>
                </div>
            </div>
        </>
    }
}

#[function_component(Login)]
pub fn login_page() -> Html {
    let username_title = use_state(|| String::from("Felhasználónév"));
    let password_title = use_state(|| String::from("Jelszó"));

    let navigator = use_navigator().unwrap();

    let username_buffer = use_state(|| String::new());
    let password_buffer = use_state(|| String::new());

    html!(
        <>
            <div id="main_island">
                <div id="nav_area">
                    <h2>{"Bejelentkezés"}</h2>
                </div>
                <TextField default_text={username_title} text_buffer={username_buffer}/>
                <TextField input_type="password" default_text={password_title} text_buffer={password_buffer}/>
                <Button label={"Regisztráció"} callback={Callback::from(|_| {
                    
                })}/>
                <div id="misc_area">
                    <a href=".">
                        <h5>{"Elfelejtette a jelszavát?"}</h5>
                    </a>
                    <a href="." onclick={Callback::from(move |_| {
                        navigator.push(&Route::Register);
                    })}>
                        <h5>{"Nincs fiókja?"}</h5>
                    </a>
                </div>
            </div>
        </>
    )
}

#[function_component(Register)]
pub fn register_page() -> Html {
    let username_title = use_state(|| String::from("Felhasználónév"));
    let password_title = use_state(|| String::from("Jelszó"));
    let username_buffer = use_state(|| String::new());
    let password_buffer = use_state(|| String::new());

    let client = Client::new();

    html!(
        <>
            <div id="main_island">
                <div id="nav_area">
                    <h2>{"Regisztráció"}</h2>
                </div>
                <TextField default_text={username_title} text_buffer={username_buffer}/>
                <TextField input_type="password" default_text={password_title} text_buffer={password_buffer}/>
                <Button label={"Regisztráció"} callback={
                    Callback::from(move |_| {
                        let post_request = client.post("http://[::1]:3004/api/register");
                    
                        wasm_bindgen_futures::spawn_local(async {
                            post_request.send().await.unwrap();
                        });
                    })}
                />
            </div>
        </>
    )
}