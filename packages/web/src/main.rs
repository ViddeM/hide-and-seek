use dioxus::prelude::*;

mod views;

use views::{HiderView, HostSetup, HostView, JoinGame, Lobby, LandingPage, NotFound, SeekerView};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    LandingPage {},
    #[route("/join")]
    JoinGame {},
    #[route("/host")]
    HostSetup {},
    #[route("/game/:game_id/lobby")]
    Lobby { game_id: String },
    #[route("/game/:game_id/seeker")]
    SeekerView { game_id: String },
    #[route("/game/:game_id/hider")]
    HiderView { game_id: String },
    #[route("/game/:game_id/host")]
    HostView { game_id: String },
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(feature = "server")]
fn main() {
    use anyhow::Context as _;
    use std::sync::Arc;

    dioxus::serve(|| async {
        let config = api::config::Config::load();
        let pool = api::db::create_pool(&config.database_url)
            .await
            .context("Failed to connect to database")?;
        let hub = api::ws::GameHub::new();
        let config = Arc::new(config);

        let router = axum::Router::new()
            .route(
                "/api/ws/{game_id}",
                axum::routing::get(api::ws::ws_handler),
            )
            .serve_dioxus_application(dioxus::server::ServeConfig::new(), App)
            .layer(axum::middleware::from_fn(api::middleware::log_middleware))
            .layer(axum::Extension(pool))
            .layer(axum::Extension(hub))
            .layer(axum::Extension(config));

        Ok(router)
    });
}
