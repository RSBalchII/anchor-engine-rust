//! Main Application Component - Anchor UI

use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <div class="app-container">
                <NavBar />
                <main class="flex-1 overflow-y-auto">
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}

#[function_component(NavBar)]
fn nav_bar() -> Html {
    html! {
        <nav class="nav-bar">
            <div class="flex flex-row gap-2" style="align-items: center;">
                <h2 style="margin: 0; font-size: 1.25rem;">{"🧠 Anchor"}</h2>
            </div>
            <ul class="nav-links">
                <li>
                    <Link<Route> to={Route::Dashboard} classes={classes!("nav-link")}>
                        {"Dashboard"}
                    </Link<Route>>
                </li>
                <li>
                    <Link<Route> to={Route::Search} classes={classes!("nav-link")}>
                        {"Search"}
                    </Link<Route>>
                </li>
                <li>
                    <Link<Route> to={Route::Chat} classes={classes!("nav-link")}>
                        {"Chat"}
                    </Link<Route>>
                </li>
                <li>
                    <Link<Route> to={Route::Settings} classes={classes!("nav-link")}>
                        {"Settings"}
                    </Link<Route>>
                </li>
            </ul>
        </nav>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Dashboard => html! { <Dashboard /> },
        Route::Search => html! { <SearchPage /> },
        Route::Chat => html! { <ChatPage /> },
        Route::Settings => html! { <SettingsPage /> },
        Route::Paths => html! { <PathsPage /> },
        Route::Quarantine => html! { <QuarantinePage /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

#[function_component(Dashboard)]
fn dashboard() -> Html {
    html! {
        <div class="dashboard animate-fade-in">
            <div style="text-align: center;">
                <h1>{"Sovereign Knowledge Engine"}</h1>
                <p class="text-dim">{"Your personal AI memory, running locally"}</p>
            </div>
            
            <div class="dashboard-grid">
                <Link<Route> to={Route::Search} classes={classes!("dashboard-card")}>
                    <h3>{"🔍 Search"}</h3>
                    <p>{"Query your knowledge base with multi-column search"}</p>
                </Link<Route>>
                
                <Link<Route> to={Route::Chat} classes={classes!("dashboard-card")}>
                    <h3>{"💬 Chat"}</h3>
                    <p>{"Chat with AI, augmented by your memory"}</p>
                </Link<Route>>
                
                <Link<Route> to={Route::Paths} classes={classes!("dashboard-card")}>
                    <h3>{"📁 Paths"}</h3>
                    <p>{"Manage watched directories"}</p>
                </Link<Route>>
                
                <Link<Route> to={Route::Quarantine} classes={classes!("dashboard-card")}>
                    <h3>{"☣️ Quarantine"}</h3>
                    <p>{"Review and cure quarantined atoms"}</p>
                </Link<Route>>
            </div>
        </div>
    }
}

#[function_component(SearchPage)]
fn search_page() -> Html {
    html! {
        <div class="glass-panel animate-fade-in" style="height: 100%;">
            <h2>{"🔍 Search"}</h2>
            <p class="text-dim">{"Multi-column search coming soon..."}</p>
            <div style="margin-top: 2rem; text-align: center;">
                <p>{"This feature is under construction."}</p>
                <p class="text-dim text-sm">{"Check back soon!"}</p>
            </div>
        </div>
    }
}

#[function_component(ChatPage)]
fn chat_page() -> Html {
    html! {
        <div class="glass-panel animate-fade-in" style="height: 100%;">
            <h2>{"💬 Chat"}</h2>
            <p class="text-dim">{"Chat interface coming soon..."}</p>
            <div style="margin-top: 2rem; text-align: center;">
                <p>{"This feature is under construction."}</p>
                <p class="text-dim text-sm">{"Check back soon!"}</p>
            </div>
        </div>
    }
}

#[function_component(PathsPage)]
fn paths_page() -> Html {
    html! {
        <div class="glass-panel animate-fade-in">
            <h2>{"📁 Path Manager"}</h2>
            <p class="text-dim">{"Configure watched directories for ingestion."}</p>
            <p>{"Coming soon..."}</p>
        </div>
    }
}

#[function_component(QuarantinePage)]
fn quarantine_page() -> Html {
    html! {
        <div class="glass-panel animate-fade-in">
            <h2>{"☣️ Quarantine"}</h2>
            <p class="text-dim">{"Review and cure quarantined atoms."}</p>
            <p>{"Coming soon..."}</p>
        </div>
    }
}

#[function_component(SettingsPage)]
fn settings_page() -> Html {
    html! {
        <div class="glass-panel animate-fade-in">
            <h2>{"⚙️ Settings"}</h2>
            <p class="text-dim">{"Configure engine connection and preferences."}</p>
            <p>{"Coming soon..."}</p>
        </div>
    }
}

#[function_component(NotFound)]
fn not_found() -> Html {
    html! {
        <div class="glass-panel animate-fade-in">
            <div class="flex flex-col flex-center" style="height: 100%; justify-content: center; align-items: center;">
                <h1>{"404"}</h1>
                <p class="text-dim">{"Page not found"}</p>
                <Link<Route> to={Route::Dashboard}>
                    <button class="button button-primary" style="margin-top: 1rem;">
                        {"← Back to Dashboard"}
                    </button>
                </Link<Route>>
            </div>
        </div>
    }
}
