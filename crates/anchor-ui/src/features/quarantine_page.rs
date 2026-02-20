//! Quarantine Page Feature

use yew::prelude::*;
use crate::components::glass_panel::GlassPanel;

#[function_component(QuarantinePage)]
pub fn quarantine_page() -> Html {
    html! {
        <GlassPanel class="quarantine-page">
            <h2>{"☣️ Quarantine - Under Construction"}</h2>
            <p class="text-dim">{"Review and cure quarantined atoms coming soon."}</p>
        </GlassPanel>
    }
}
