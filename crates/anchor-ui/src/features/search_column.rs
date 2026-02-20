//! Search Column Feature

use wasm_bindgen::JsCast;
use yew::prelude::*;
use crate::api::{ApiClient, SearchRequest, SearchResponse};
use crate::components::glass_panel::GlassPanel;
use crate::components::button::Button;
use crate::components::input::Input;

#[derive(Properties, PartialEq, Clone)]
pub struct SearchColumnProps {
    pub id: u32,
    pub on_remove: Callback<u32>,
}

#[function_component(SearchColumn)]
pub fn search_column(props: &SearchColumnProps) -> Html {
    let query = use_state(|| String::new());
    let results = use_state(|| Vec::<SearchResponse>::new());
    let loading = use_state(|| false);
    let api_client = ApiClient::default();

    let query_clone = query.clone();
    let onquery = Callback::from(move |value: String| {
        query_clone.set(value);
    });

    let loading_clone = loading.clone();
    let results_clone = results.clone();
    let query_for_search = query.clone();

    let onsearch = {
        let api = api_client.clone();
        Callback::from(move |_| {
            let q = (*query_for_search).clone();
            if q.trim().is_empty() {
                return;
            }

            loading_clone.set(true);

            let api_clone = api.clone();
            let results_clone = results_clone.clone();
            let loading_clone = loading_clone.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let request = SearchRequest {
                    query: q,
                    max_chars: Some(2048),
                    token_budget: Some(2048),
                    buckets: None,
                    tags: None,
                    include_code: None,
                };

                match api_clone.search(request).await {
                    Ok(response) => {
                        results_clone.set(vec![response]);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Search error: {}", e).into());
                    }
                }
                loading_clone.set(false);
            });
        })
    };

    let onremove = {
        let id = props.id;
        let callback = props.on_remove.clone();
        Callback::from(move |_| callback.emit(id))
    };

    let result_items = (*results).iter().map(|result| {
        html! {
            <div class="search-result-item">
                <p class="result-content">{ &result.context[..result.context.len().min(200)] }</p>
            </div>
        }
    }).collect::<Html>();

    html! {
        <GlassPanel class="search-column">
            <div class="search-column-header">
                <h3>{ format!("Search {}", props.id) }</h3>
                <Button onclick={onremove} class="remove-btn">{"✕"}</Button>
            </div>

            <div class="search-input-container">
                <Input
                    value={(*query).clone()}
                    onchange={onquery}
                    placeholder={"Search your knowledge..."}
                />
                <Button onclick={onsearch} disabled={*loading}>{"🔍"}</Button>
            </div>

            <div class="search-results">
                if *loading {
                    <div class="loading">{"Searching..."}</div>
                } else if (*results).is_empty() {
                    <div class="text-dim text-sm">{"No results yet. Try a search!"}</div>
                } else {
                    {result_items}
                }
            </div>
        </GlassPanel>
    }
}
