//! Chat Interface Feature

use yew::prelude::*;
use crate::components::glass_panel::GlassPanel;

#[derive(Properties, PartialEq, Clone)]
pub struct ChatInterfaceProps {
    #[prop_or_default]
    pub model: String,
}

#[function_component(ChatInterface)]
pub fn chat_interface(_props: &ChatInterfaceProps) -> Html {
    html! {
        <GlassPanel class="chat-interface">
            <h2>{"💬 Chat - Under Construction"}</h2>
            <p class="text-dim">{"Chat interface with RAG context coming soon."}</p>
        </GlassPanel>
    }
}
