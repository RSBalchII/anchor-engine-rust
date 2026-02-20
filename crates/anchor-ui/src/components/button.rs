//! Button Component

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ButtonProps {
    pub children: Children,
    #[prop_or_default]
    pub onclick: Callback<()>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: String,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let onclick = {
        let callback = props.onclick.clone();
        Callback::from(move |_| callback.emit(()))
    };

    let mut classes = Classes::from("button");
    if props.disabled {
        classes.push("button-disabled");
    }
    classes.push(props.class.clone());

    html! {
        <button
            class={classes}
            onclick={onclick}
            disabled={props.disabled}
            style={props.style.clone()}
        >
            { for props.children.iter() }
        </button>
    }
}
