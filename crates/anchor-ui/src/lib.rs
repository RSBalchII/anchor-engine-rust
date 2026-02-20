//! Anchor UI - Main Application Module

pub mod app;
pub mod routes;
pub mod api;
pub mod state;

pub mod components {
    pub mod button;
    pub mod input;
    pub mod glass_panel;
    pub mod badge;
    pub mod loading;
}

pub mod features {
    pub mod search_column;
    pub mod chat_interface;
    pub mod path_manager;
    pub mod quarantine_page;
    pub mod settings;
}

pub use app::App;
pub use routes::Route;
pub use features::search_column::SearchColumn;
pub use features::chat_interface::ChatInterface;
pub use features::path_manager::PathManager;
pub use features::quarantine_page::QuarantinePage;
pub use features::settings::Settings;
