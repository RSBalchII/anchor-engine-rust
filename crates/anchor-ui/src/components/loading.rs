//! Loading Spinner Component

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct LoadingProps {
    #[prop_or_default]
    pub text: String,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(Loading)]
pub fn loading(props: &LoadingProps) -> Html {
    html! {
        <div class={classes!("loading", props.class.clone())}>
            <div class="spinner"></div>
            if !props.text.is_empty() {
                <p class="text-dim text-sm">{ &props.text }</p>
            }
        </div>
    }
}
