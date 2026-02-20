//! Path Manager Feature

use yew::prelude::*;
use crate::components::glass_panel::GlassPanel;

#[function_component(PathManager)]
pub fn path_manager() -> Html {
    html! {
        <GlassPanel class="path-manager">
            <h2>{"📁 Path Manager - Under Construction"}</h2>
            <p class="text-dim">{"Configure watched directories for ingestion coming soon."}</p>
        </GlassPanel>
    }
}
