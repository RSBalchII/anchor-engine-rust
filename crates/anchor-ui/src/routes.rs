//! Application Routes

use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Dashboard,

    #[at("/search")]
    Search,

    #[at("/chat")]
    Chat,

    #[at("/paths")]
    Paths,

    #[at("/quarantine")]
    Quarantine,

    #[at("/settings")]
    Settings,

    #[not_found]
    #[at("/404")]
    NotFound,
}
