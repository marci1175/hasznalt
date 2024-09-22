use yew::{html, Component, Properties};

pub struct Searchbar;

#[derive(Debug, PartialEq, Properties)]
pub struct SearchbarProperties {
    pub default_text: String,
}

impl Component for Searchbar {
    type Message = ();

    type Properties = SearchbarProperties;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html!(
            <input type="text" placeholder={ctx.props().default_text.clone()}/>
        )    
    }
}