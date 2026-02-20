//! Settings Feature

use yew::prelude::*;
use crate::components::glass_panel::GlassPanel;

#[function_component(Settings)]
pub fn settings() -> Html {
    html! {
        <GlassPanel class="settings">
            <h2>{"⚙️ Settings - Under Construction"}</h2>
            <p class="text-dim">{"Configure engine connection and preferences coming soon."}</p>
        </GlassPanel>
    }
}
