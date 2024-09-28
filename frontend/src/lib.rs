use web_sys::HtmlTextAreaElement;
use yew::{virtual_dom::VNode, Callback, Component, InputEvent, MouseEvent, Properties, TargetCast, UseStateHandle};
use yew::html;
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

pub struct Button;

#[derive(Debug, PartialEq, Properties)]
pub struct SearchButtonProperties
{
    pub label: VNode,

    pub callback: Callback<MouseEvent>, 
}

impl Component for Button
{
    type Message = ();

    type Properties = SearchButtonProperties;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html!(
            <button onclick={ctx.props().callback.clone()}>{ctx.props().label.clone()}</button>
        )
    }
}