//! Badge Component

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct BadgeProps {
    pub label: String,
    #[prop_or_default]
    pub color: String,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: String,
}

#[function_component(Badge)]
pub fn badge(props: &BadgeProps) -> Html {
    let mut classes = Classes::from("badge");
    classes.push(props.class.clone());

    let background = if !props.color.is_empty() {
        props.color.clone()
    } else {
        "rgba(6, 182, 212, 0.2)".to_string()
    };

    html! {
        <span class={classes} style={format!("background: {}; {}", background, props.style)}>
            { props.label.clone() }
        </span>
    }
}
