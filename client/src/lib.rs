use web_sys::HtmlTextAreaElement;
use yew::{html, Callback, Component, InputEvent, Properties, TargetCast, UseStateHandle};

pub struct Searchbar;

#[derive(Debug, PartialEq, Properties)]
pub struct SearchbarProperties {
    pub default_text: UseStateHandle<String>,
    pub text_buffer: UseStateHandle<String>,
}

pub enum SearchBarMessage {
    ValueUpdate(String),
}

impl Component for Searchbar {
    type Message = SearchBarMessage;

    type Properties = SearchbarProperties;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let text_edit_value_callback: Callback<InputEvent> =
            ctx.link().callback(move |event: InputEvent| {
                let html_elem: HtmlTextAreaElement = event.target_unchecked_into();

                SearchBarMessage::ValueUpdate(html_elem.value())
            });

        html!(
            <input type="text" placeholder={ctx.props().default_text.to_string()} oninput={text_edit_value_callback}/>
        )    
    }
    
    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SearchBarMessage::ValueUpdate(msg) => {
                ctx.props().text_buffer.set(msg);
            },
        }

        true
    }
}

pub struct SearchButton;

#[derive(Debug, PartialEq, Properties)]
pub struct SearchButtonProperties
{
    pub label: UseStateHandle<String>,
}

impl Component for SearchButton
{
    type Message = ();

    type Properties = SearchButtonProperties;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let onclick_callback = Callback::from(move |event: yew::MouseEvent| {

        });

        html!(
            <button onclick={onclick_callback}><img src="public\\search.svg" height=20/></button>
        )
    }
}