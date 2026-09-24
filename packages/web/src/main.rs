#[cfg(feature = "server")]
use api::cli::Cli;
#[cfg(feature = "server")]
use clap::Parser;
use dioxus::prelude::*;

mod views;

use uuid::Uuid;
use views::{
    EditMapBoundary, EditMapConfirm, EditMapInfo, EditMapTransit, GameView, HostSetup,
    LandingPage, NewMap,
};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    LandingPage {},
    #[route("/host")]
    HostSetup {},
    // Map creation wizard — each step is its own URL
    #[route("/maps/new")]
    NewMap {},
    #[route("/maps/:id/edit")]
    EditMapInfo { id: Uuid },
    #[route("/maps/:id/edit/boundary")]
    EditMapBoundary { id: Uuid },
    #[route("/maps/:id/edit/transit")]
    EditMapTransit { id: Uuid },
    #[route("/maps/:id/edit/confirm")]
    EditMapConfirm { id: Uuid },
    #[route("/game/:game_id")]
    GameView { game_id: Uuid },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        ErrorBoundary {
            handle_error: |ctx: ErrorContext| rsx! {
                main { class: "error-page",
                    if let Some(e) = ctx.error() {
                        p { class: "form-error", "{e}" }
                    }
                    a { href: "/", "← Back to start" }
                }
            },
            SuspenseBoundary {
                fallback: |_| rsx! { main { class: "loading", p { "Loading…" } } },
                Router::<Route> {}
            }
        }
    }
}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(App);
}

#[cfg(feature = "server")]
fn main() {
    dotenvy::dotenv().ok();

    dioxus::serve(|| async {
        let args = Cli::parse();

        let pool = api::db::create_pool(&args.database_url)
            .await
            .context("Failed to connect to database")?;

        let router = axum::Router::new()
            .serve_dioxus_application(dioxus::server::ServeConfig::new(), App)
            .layer(axum::middleware::from_fn(
                api::middleware::logging::log_middleware,
            ))
            .layer(axum::Extension(pool));

        Ok(router)
    });
}
