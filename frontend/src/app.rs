use std::str::FromStr;

use frontend::{Button, NewAccount, TextField};
use js_sys::JsString;
use reqwest::Client;
use web_sys::console;
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
    let search_buffer = use_state(String::new);

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

    let username_buffer = use_state(String::new);
    let password_buffer = use_state(String::new);

    html!(
        <>
            <div id="login_island">
                <div id="login_nav_area">
                    <h2>{"Bejelentkezés"}</h2>
                </div>
                <TextField default_text={username_title} text_buffer={username_buffer.clone()}/>
                <TextField input_type="password" default_text={password_title} text_buffer={password_buffer.clone()}/>

                <Button label={"Bejelentkezés"} callback={Callback::from(move |_| {
                    let client = Client::new();

                    let post_request = client.post("http://[::1]:3004/api/login".to_string());
                    let password_buffer = password_buffer.clone();
                    let username_buffer = username_buffer.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let request = post_request
                            .header("Content-Type", "application/json")
                            .body(
                                serde_json::to_string(&NewAccount {
                                    password: password_buffer.to_string(),
                                    username: username_buffer.to_string(),
                                }).unwrap()
                            )
                            .send()
                            .await
                            .unwrap();

                        console::debug_1(&JsString::from_str(&request.text().await.unwrap()).unwrap());
                    });
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
    let username_buffer = use_state(String::new);
    let password_buffer = use_state(String::new);

    html!(
        <>
            <div id="register_area">
                <div id="register_nav_area">
                    <h2>{"Regisztráció"}</h2>
                </div>
                <TextField default_text={username_title} text_buffer={username_buffer.clone()}/>
                <TextField input_type="password" default_text={password_title} text_buffer={password_buffer.clone()}/>
                <Button label={"Regisztráció"} callback={
                    Callback::from(move |_| {
                        let client = Client::new();

                        let post_request = client.post("http://[::1]:3004/api/register".to_string());
                        let password_buffer = password_buffer.clone();
                        let username_buffer = username_buffer.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            post_request
                                .header("Content-Type", "application/json")
                                .body(
                                    serde_json::to_string(&NewAccount {
                                        password: password_buffer.to_string(),
                                        username: username_buffer.to_string(),
                                    }).unwrap()
                                )
                                .send()
                                .await
                                .unwrap();
                        });
                    })}
                />
            </div>
        </>
    )
}
