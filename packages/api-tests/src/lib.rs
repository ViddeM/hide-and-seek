pub mod seed;

use api::endpoints::game::CreateGameRequest;
use api::endpoints::maps::CreateMapRequest;
use dioxus::prelude::*;
use testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

/// A running test server and the resources that keep it alive.
/// Drop this at the end of a test to stop both the HTTP server and the DB container.
pub struct TestServer {
    pub base_url: String,
    pub pool: sqlx::PgPool,
    // Keeping this field drops (and stops) the container when TestServer is dropped.
    _container: ContainerAsync<Postgres>,
}

/// Start a fresh Postgres container, run migrations, and spin up the API server
/// on a random port. Each call produces a fully isolated environment.
pub async fn spawn_test_server() -> TestServer {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .expect("Failed to start Postgres container");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let pool = api::db::create_pool(&db_url)
        .await
        .expect("Failed to connect to test database and run migrations");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to a random port");
    let addr = listener.local_addr().unwrap();

    let router = axum::Router::new()
        .register_server_functions()
        .with_state(dioxus_server::FullstackState::headless())
        .layer(axum::Extension(pool.clone()));

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    TestServer {
        base_url: format!("http://{addr}"),
        pool,
        _container: container,
    }
}

/// The #[post] macro wraps client-side arguments in a struct keyed by the
/// argument name, so the wire format is `{"request": {…}}` not the struct itself.
pub fn game_body(request: CreateGameRequest) -> serde_json::Value {
    serde_json::json!({ "request": request })
}

pub fn map_body(request: CreateMapRequest) -> serde_json::Value {
    serde_json::json!({ "request": request })
}
