use frontend::{
    get_cookie, request_account_lookup_from_cookie, request_account_lookup_from_id, AccountCredentials, AccountLookup, AccountPageProperties, AuthorizedUser, Button, TextField
};
use reqwest::Client;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::{hooks::use_navigator, BrowserRouter, Routable, Switch};

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[at("/")]
    MainPage,
    #[at("/register")]
    Register,
    #[at("/login")]
    Login,
    #[at("/account/:id")]
    IdLookup { id: i32 },
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::MainPage => html! { <Main /> },
        Route::Register => html! { <Register /> },
        Route::Login => html! { <Login /> },
        Route::IdLookup { id } => html! { <Account id={id}/> },
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
    let requested_account: UseStateHandle<Option<AccountLookup>> = use_state_eq(|| None);

    // The cookie will be inspected by the server we only check if it exists
    if get_cookie("session_id").is_some() {
        let requested_account_clone = requested_account.clone();

        spawn_local(async move {
            requested_account_clone.set(request_account_lookup_from_cookie().await.ok());
        });
    }

    html! {
        <>
            <div id="navigation">
                {
                    if let Some(requested_account) = (*requested_account).clone() {
                        html!(
                            <>
                                <h5>{ format!("Bejelentkezve mint, {}", requested_account.username) }</h5>
                                <Button label={ "Fiókom" }
                                    callback={
                                        let navigator = navigator.clone();
                                        Callback::from(move |_| {
                                            navigator.push(&Route::IdLookup { id: requested_account.id });
                                        })
                                    }
                                />
                            </>
                        )
                    }
                    else {
                        html!(<>
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
                        </>)
                    }
                }
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
    let navigator = use_navigator().unwrap();

    let username_title = use_state(|| String::from("Felhasználónév"));
    let password_title = use_state(|| String::from("Jelszó"));
    let username_buffer = use_state(String::new);
    let password_buffer = use_state(String::new);

    let login_success: UseStateHandle<Option<bool>> = use_state_eq(|| None);
    let login_success_clone = login_success.clone();

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
                    let login_success = login_success.clone();
                    let post_request = client.post("http://[::1]:3004/api/login".to_string());
                    let password_buffer = password_buffer.clone();
                    let username_buffer = username_buffer.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let request = post_request
                            .header("Content-Type", "application/json")
                            .body(
                                serde_json::to_string(&AccountCredentials {
                                    passw: password_buffer.to_string(),
                                    username: username_buffer.to_string(),
                                }).unwrap()
                            )
                            .send()
                            .await
                            .unwrap();

                        login_success.set(Some(request.status().is_success()));
                    });
                })}/>

                {
                    if let Some(login_success) = (*login_success_clone).clone() {
                        if login_success {
                            navigator.push(&Route::MainPage);

                            html!(
                                <div id="success_prompt">
                                    <h5>{"Login successful! Redirecting. . ."}</h5>
                                </div>
                            )
                        }
                        else {
                            html!(
                                <div id="fail_prompt">
                                    <h5>{"Invalid username or password!"}</h5>
                                </div>
                            )
                        }
                    }
                    else {
                        html!()
                    }
                }

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
                                    serde_json::to_string(&AccountCredentials {
                                        passw: password_buffer.to_string(),
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

#[function_component(Account)]
pub fn account_page(AccountPageProperties { id }: &AccountPageProperties) -> Html {
    let requested_account: UseStateHandle<AccountLookup> = use_state_eq(AccountLookup::default);

    let requested_account_clone = requested_account.clone();

    let id_clone = *id;

    spawn_local(async move {
        requested_account_clone.set(request_account_lookup_from_id(id_clone).await.unwrap());
    });

    html!(
        <div id="username_title">
            <center>
                <h1>
                    { requested_account.username.clone() }
                </h1>
                <h3>
                    { requested_account.created_at.to_string() }
                </h3>
            </center>
        </div>
    )
}