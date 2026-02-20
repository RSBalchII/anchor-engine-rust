//! GlassPanel Component - Glassmorphism container

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct GlassPanelProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: String,
}

#[function_component(GlassPanel)]
pub fn glass_panel(props: &GlassPanelProps) -> Html {
    let mut classes = Classes::from("glass-panel");
    classes.push(props.class.clone());

    html! {
        <div class={classes} style={props.style.clone()}>
            { for props.children.iter() }
        </div>
    }
}
