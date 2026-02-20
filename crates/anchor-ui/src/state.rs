//! Global Application State

use yew::prelude::*;
use crate::api::ApiClient;

/// Global application state using Yew's Context API
#[derive(Clone, Debug)]
pub struct AppState {
    pub api_client: ApiClient,
    pub dark_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            api_client: ApiClient::default(),
            dark_mode: true,
        }
    }
}

/// Context type for state
pub type StateContext = UseStateHandle<AppState>;
