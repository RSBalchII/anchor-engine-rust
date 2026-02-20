//! Input Component

use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct InputProps {
    #[prop_or_default]
    pub value: String,
    #[prop_or_default]
    pub onchange: Callback<String>,
    #[prop_or_default]
    pub placeholder: String,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: String,
    #[prop_or_default]
    pub input_type: String,
}

#[function_component(Input)]
pub fn input(props: &InputProps) -> Html {
    let onchange = {
        let callback = props.onchange.clone();
        Callback::from(move |e: Event| {
            let target: web_sys::HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            callback.emit(target.value())
        })
    };

    let mut classes = Classes::from("input");
    classes.push(props.class.clone());

    html! {
        <input
            class={classes}
            type={props.input_type.clone()}
            value={props.value.clone()}
            onchange={onchange}
            disabled={props.disabled}
            placeholder={props.placeholder.clone()}
            style={props.style.clone()}
        />
    }
}
