use std::fmt::Display;

use serde::{Deserialize, Serialize};
use web_sys::HtmlTextAreaElement;
use yew::html;
use yew::{
    virtual_dom::VNode, Callback, Component, InputEvent, MouseEvent, Properties, TargetCast,
    UseStateHandle,
};

/// ```TextField``` component definition
pub struct TextField;

/// ```TextField``` properties.
#[derive(Debug, PartialEq, Properties)]
pub struct TextFieldProperties {
    /// The placeholder text this ```TextField``` should display when it's empty.
    pub default_text: UseStateHandle<String>,

    /// The text buffer is where the entered text is stored at
    pub text_buffer: UseStateHandle<String>,

    /// Specify the input type of this ```TextField```. (This field is optional, the default is "text")
    #[prop_or_default]
    pub input_type: String,

    /// The id of the ```TextField``` (This field is optional)
    #[prop_or_default]
    pub id: String,
}

impl Default for TextFieldProperties {
    fn default() -> Self {
        Self {
            input_type: String::from("text"),
            id: String::new(),
            default_text: unimplemented!("A value must be passed in for this field"),
            #[allow(unreachable_code)]
            text_buffer: unimplemented!("A value must be passed in for this field"),
        }
    }
}

/// The ```TextField``` update's messages
pub enum TextFieldMessage {
    /// This get sent when the TextField has it's buffer updated
    ValueUpdate(String),
}

impl Component for TextField {
    type Message = TextFieldMessage;

    type Properties = TextFieldProperties;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let text_edit_value_callback: Callback<InputEvent> =
            ctx.link().callback(move |event: InputEvent| {
                let html_elem: HtmlTextAreaElement = event.target_unchecked_into();

                TextFieldMessage::ValueUpdate(html_elem.value())
            });

        html!(
            <input id={ctx.props().id.clone()} type={ctx.props().input_type.clone()} placeholder={ctx.props().default_text.to_string()} oninput={text_edit_value_callback}/>
        )
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TextFieldMessage::ValueUpdate(msg) => {
                ctx.props().text_buffer.set(msg);
            }
        }

        true
    }
}

/// ```Button``` component definition
pub struct Button;

/// ```Button``` properties.
#[derive(Debug, PartialEq, Properties)]
pub struct ButtonProperties {
    /// Label for this button, this is an html element.
    pub label: VNode,

    /// Callback is called when the button is called
    pub callback: Callback<MouseEvent>,

    /// The id of the button (This field is optional)
    #[prop_or_default]
    pub id: String,
}

impl Component for Button {
    type Message = ();

    type Properties = ButtonProperties;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html!(
            <button id={ctx.props().id.clone()} onclick={ctx.props().callback.clone()}>{ctx.props().label.clone()}</button>
        )
    }
}

#[derive(Serialize)]
pub struct NewAccount {
    pub username: String,
    pub passw: String,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AccountLookup {
    /// The username of the requested user
    pub username: String,
    /// The UUID of the requested user
    pub id: i32,
    /// The timestamp taken when the account was created
    pub created_at: chrono::NaiveDate,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthorizedUser {
    pub client_signature: String,
    pub session_id: String,
    pub account_id: i32,
}

impl Display for AuthorizedUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct AccountPageProperties {
    pub id: i32,
}
