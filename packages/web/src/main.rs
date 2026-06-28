#[cfg(feature = "server")]
use api::cli::Cli;
#[cfg(feature = "server")]
use clap::Parser;
use dioxus::prelude::*;

mod views;

use views::{HostSetup, LandingPage};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    LandingPage {},
    // #[route("/join")]
    // JoinGame {},
    #[route("/host")]
    HostSetup {},
    // #[route("/game/:game_id/lobby")]
    // Lobby { game_id: String },
    // #[route("/game/:game_id/seeker")]
    // SeekerView { game_id: String },
    // #[route("/game/:game_id/hider")]
    // HiderView { game_id: String },
    // #[route("/game/:game_id/host")]
    // HostView { game_id: String },
    // #[route("/:..segments")]
    // NotFound { segments: Vec<String> },
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
    dotenvy::dotenv().ok();

    dioxus::serve(|| async {
        let args = Cli::parse();

        let pool = api::db::create_pool(&args.database_url)
            .await
            .context("Failed to connect to database")?;

        // let hub = api::ws::GameHub::new();

        let router = axum::Router::new()
            .serve_dioxus_application(dioxus::server::ServeConfig::new(), App)
            .layer(axum::middleware::from_fn(
                api::middleware::logging::log_middleware,
            ))
            .layer(axum::Extension(pool));

        Ok(router)
    });
}
