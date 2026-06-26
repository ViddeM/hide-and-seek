You are an expert [0.7 Dioxus](https://dioxuslabs.com/learn/0.7) assistant. Dioxus 0.7 changes every api in dioxus. Only use this up to date documentation. `cx`, `Scope`, and `use_state` are gone

Provide concise code examples with detailed descriptions

# Dioxus Dependency

You can add Dioxus to your `Cargo.toml` like this:

```toml
[dependencies]
dioxus = { version = "0.7.1" }

[features]
default = ["web", "webview", "server"]
web = ["dioxus/web"]
webview = ["dioxus/desktop"]
server = ["dioxus/server"]
```

# Launching your application

You need to create a main function that sets up the Dioxus runtime and mounts your root component.

```rust
use dioxus::prelude::*;

fn main() {
	dioxus::launch(App);
}

#[component]
fn App() -> Element {
	rsx! { "Hello, Dioxus!" }
}
```

Then serve with `dx serve`:

```sh
curl -sSL http://dioxus.dev/install.sh | sh
dx serve
```

# UI with RSX

```rust
rsx! {
	div {
		class: "container", // Attribute
		color: "red", // Inline styles
		width: if condition { "100%" }, // Conditional attributes
		"Hello, Dioxus!"
	}
	// Prefer loops over iterators
	for i in 0..5 {
		div { "{i}" } // use elements or components directly in loops
	}
	if condition {
		div { "Condition is true!" } // use elements or components directly in conditionals
	}

	{children} // Expressions are wrapped in brace
	{(0..5).map(|i| rsx! { span { "Item {i}" } })} // Iterators must be wrapped in braces
}
```

# Assets

The asset macro can be used to link to local files to use in your project. All links start with `/` and are relative to the root of your project.

```rust
rsx! {
	img {
		src: asset!("/assets/image.png"),
		alt: "An image",
	}
}
```

## Styles

The `document::Stylesheet` component will inject the stylesheet into the `<head>` of the document

```rust
rsx! {
	document::Stylesheet {
		href: asset!("/assets/styles.css"),
	}
}
```

# Components

Components are the building blocks of apps

* Component are functions annotated with the `#[component]` macro.
* The function name must start with a capital letter or contain an underscore.
* A component re-renders only under two conditions:
	1.  Its props change (as determined by `PartialEq`).
	2.  An internal reactive state it depends on is updated.

```rust
#[component]
fn Input(mut value: Signal<String>) -> Element {
	rsx! {
		input {
            value,
			oninput: move |e| {
				*value.write() = e.value();
			},
			onkeydown: move |e| {
				if e.key() == Key::Enter {
					value.write().clear();
				}
			},
		}
	}
}
```

Each component accepts function arguments (props)

* Props must be owned values, not references. Use `String` and `Vec<T>` instead of `&str` or `&[T]`.
* Props must implement `PartialEq` and `Clone`.
* To make props reactive and copy, you can wrap the type in `ReadOnlySignal`. Any reactive state like memos and resources that read `ReadOnlySignal` props will automatically re-run when the prop changes.

# State

A signal is a wrapper around a value that automatically tracks where it's read and written. Changing a signal's value causes code that relies on the signal to rerun.

## Local State

The `use_signal` hook creates state that is local to a single component. You can call the signal like a function (e.g. `my_signal()`) to clone the value, or use `.read()` to get a reference. `.write()` gets a mutable reference to the value.

Use `use_memo` to create a memoized value that recalculates when its dependencies change. Memos are useful for expensive calculations that you don't want to repeat unnecessarily.

```rust
#[component]
fn Counter() -> Element {
	let mut count = use_signal(|| 0);
	let mut doubled = use_memo(move || count() * 2); // doubled will re-run when count changes because it reads the signal

	rsx! {
		h1 { "Count: {count}" } // Counter will re-render when count changes because it reads the signal
		h2 { "Doubled: {doubled}" }
		button {
			onclick: move |_| *count.write() += 1, // Writing to the signal rerenders Counter
			"Increment"
		}
		button {
			onclick: move |_| count.with_mut(|count| *count += 1), // use with_mut to mutate the signal
			"Increment with with_mut"
		}
	}
}
```

## Context API

The Context API allows you to share state down the component tree. A parent provides the state using `use_context_provider`, and any child can access it with `use_context`

```rust
#[component]
fn App() -> Element {
	let mut theme = use_signal(|| "light".to_string());
	use_context_provider(|| theme); // Provide a type to children
	rsx! { Child {} }
}

#[component]
fn Child() -> Element {
	let theme = use_context::<Signal<String>>(); // Consume the same type
	rsx! {
		div {
			"Current theme: {theme}"
		}
	}
}
```

# Async

For state that depends on an asynchronous operation (like a network request), Dioxus provides a hook called `use_resource`. This hook manages the lifecycle of the async task and provides the result to your component.

* The `use_resource` hook takes an `async` closure. It re-runs this closure whenever any signals it depends on (reads) are updated
* The `Resource` object returned can be in several states when read:
1. `None` if the resource is still loading
2. `Some(value)` if the resource has successfully loaded

```rust
let mut dog = use_resource(move || async move {
	// api request
});

match dog() {
	Some(dog_info) => rsx! { Dog { dog_info } },
	None => rsx! { "Loading..." },
}
```

# Routing

All possible routes are defined in a single Rust `enum` that derives `Routable`. Each variant represents a route and is annotated with `#[route("/path")]`. Dynamic Segments can capture parts of the URL path as parameters by using `:name` in the route string. These become fields in the enum variant.

The `Router<Route> {}` component is the entry point that manages rendering the correct component for the current URL.

You can use the `#[layout(NavBar)]` to create a layout shared between pages and place an `Outlet<Route> {}` inside your layout component. The child routes will be rendered in the outlet.

```rust
#[derive(Routable, Clone, PartialEq)]
enum Route {
	#[layout(NavBar)] // This will use NavBar as the layout for all routes
		#[route("/")]
		Home {},
		#[route("/blog/:id")] // Dynamic segment
		BlogPost { id: i32 },
}

#[component]
fn NavBar() -> Element {
	rsx! {
		a { href: "/", "Home" }
		Outlet<Route> {} // Renders Home or BlogPost
	}
}

#[component]
fn App() -> Element {
	rsx! { Router::<Route> {} }
}
```

```toml
dioxus = { version = "0.7.1", features = ["router"] }
```

# Fullstack

Fullstack enables server rendering and ipc calls. It uses Cargo features (`server` and a client feature like `web`) to split the code into a server and client binaries.

```toml
dioxus = { version = "0.7.1", features = ["fullstack"] }
```

## Server Functions

Use the `#[post]` / `#[get]` macros to define an `async` function that will only run on the server. On the server, this macro generates an API endpoint. On the client, it generates a function that makes an HTTP request to that endpoint.

```rust
#[post("/api/double/:path/&query")]
async fn double_server(number: i32, path: String, query: i32) -> Result<i32, ServerFnError> {
	tokio::time::sleep(std::time::Duration::from_secs(1)).await;
	Ok(number * 2)
}
```

## Hydration

Hydration is the process of making a server-rendered HTML page interactive on the client. The server sends the initial HTML, and then the client-side runs, attaches event listeners, and takes control of future rendering.

### Errors
The initial UI rendered by the component on the client must be identical to the UI rendered on the server.

* Use the `use_server_future` hook instead of `use_resource`. It runs the future on the server, serializes the result, and sends it to the client, ensuring the client has the data immediately for its first render.
* Any code that relies on browser-specific APIs (like accessing `localStorage`) must be run *after* hydration. Place this code inside a `use_effect` hook.

# Database (sqlx)

The connection pool lives in an Axum `Extension<PgPool>` injected at server startup. Inside server functions, extract it with `FullstackContext::extract`:

```rust
use axum::extract::Extension;
use sqlx::PgPool;

let pool: Extension<PgPool> = FullstackContext::extract().await
    .map_err(|e| AppError::Internal(e.to_string()))?;
let pool = pool.0;
```

Run migrations at startup (before serving):
```rust
sqlx::migrate!("./migrations").run(&pool).await.expect("migrations");
```

Use `sqlx::query_as!` with the struct name and SQL. If type inference fails on `query_scalar`, add explicit type: `sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM ...").fetch_one(&pool).await?`.

`FromRow` impls for server-only types must be gated: `#[cfg(feature = "server")] #[derive(sqlx::FromRow)]`.

# JWT in Server Functions

Claims are packed into a JWT signed with HS256 and delivered via an HTTP-only `Set-Cookie: auth=<token>` response header. To set a cookie from a server function, extract the `ResponseParts` extension:

```rust
use axum::http::header::{COOKIE, SET_COOKIE};
use dioxus::fullstack::FullstackContext;

let parts: axum::response::Parts = FullstackContext::extract().await?;
parts.headers.append(SET_COOKIE, cookie_value.parse().unwrap());
```

To read the cookie, extract `axum_extra::extract::CookieJar`:
```rust
let jar: axum_extra::extract::CookieJar = FullstackContext::extract().await?;
let token = jar.get("auth").map(|c| c.value().to_string());
```

`get_session()` returns `Result<Option<SessionInfo>, ServerFnError>` — `None` means no valid cookie, not an error.

# WebSockets

WebSocket upgrades cannot be handled by server functions (they need `WebSocketUpgrade` extractor). Add a dedicated Axum route alongside the Dioxus router:

```rust
let router = axum::Router::new()
    .route("/api/ws/:game_id", axum::routing::get(api::ws::ws_handler))
    .serve_dioxus_application(ServeConfig::new(), App)
    .layer(axum::Extension(pool))
    .layer(axum::Extension(hub));
```

The hub uses `tokio::sync::broadcast` channels per game. The handler upgrades, reads the `auth` cookie for role, then subscribes and filters messages by role.

On the client side, open a WebSocket via JS interop (not a native Rust API):
```rust
use_effect(move || {
    let js = format!("var ws=new WebSocket('ws://'+location.host+'/api/ws/{id}'); ...");
    let _ = document::eval(&js);
});
```

**axum 0.8 WebSocket change**: `Message::Text` now holds `Utf8Bytes` instead of `String`. When sending: `Message::Text(string.into())`. When receiving: pattern binds `Utf8Bytes` — use `.as_str()` to parse with serde.

# Configuration (clap + dotenvy)

```rust
#[derive(clap::Parser)]
pub struct Config {
    #[arg(env = "DATABASE_URL")]
    pub database_url: String,
    #[arg(env = "JWT_SECRET")]
    pub jwt_secret: String,
    #[arg(env = "HOST", default_value = "0.0.0.0:8080")]
    pub host: String,
}

impl Config {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();
        Self::parse()
    }
}
```

Wrap in `Arc<Config>` and inject as `axum::Extension(config)`. Extract in server functions the same way as `Extension<PgPool>`.

# Logging

Use `env_logger` (no `tracing` needed). Inject middleware via `axum::middleware::from_fn`:

```rust
pub async fn log_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    log::info!("→ {method} {path}");
    let start = Instant::now();
    let response = next.run(req).await;
    let status = response.status();
    let ms = start.elapsed().as_millis();
    match status.is_server_error() {
        true  => log::error!("← {status} {ms}ms {method} {path}"),
        false if status.is_client_error() => log::warn!("← {status} {ms}ms {method} {path}"),
        _     => log::info!("← {status} {ms}ms {method} {path}"),
    }
    response
}
```

`tower-http`'s `TraceLayer` requires `tracing::Span` from the `tracing` crate — avoid it unless `tracing` is already a dependency.

# Leaflet.js via eval()

Inject Leaflet from CDN using `document::Link` / `document::Script` in RSX. Initialise the map with `use_effect` (runs only after hydration):

```rust
use_effect(move || {
    let js = format!(r#"
        (function(){{
            if(window._map) return;
            var m = L.map('map-div').fitBounds([[{sw},{sw_lng}],[{ne},{ne_lng}]]);
            L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png').addTo(m);
            window._map = m; window._zones = {{}};
        }})();
    "#);
    let _ = document::eval(&js);
});
```

Sync a `Signal<Vec<Zone>>` to Leaflet circles in a second `use_effect` that reads the signal:
```rust
use_effect(move || {
    let snap = zones.read().clone();
    // build JS that removes stale circles and adds new ones
    let _ = document::eval(&js);
});
```

The `#leaflet-map` div must have an explicit height (e.g. `style: "height:100%"`).

# Custom Axum Server Launch (Dioxus 0.7)

Use `dioxus::serve(|| async { ... })` — the closure returns `Result<Router, anyhow::Error>`. Chain `.serve_dioxus_application(ServeConfig::new(), App)` on the router (from `DioxusRouterExt`):

```rust
dioxus::serve(|| async {
    let pool = create_pool(&config.database_url).await?;
    let router = axum::Router::new()
        .route("/api/ws/{game_id}", axum::routing::get(ws_handler))
        .serve_dioxus_application(dioxus::server::ServeConfig::new(), App)
        .layer(axum::middleware::from_fn(log_middleware))
        .layer(axum::Extension(pool));
    Ok(router)
});
```

`ServeConfig` is in `dioxus::server` (under `#[cfg(feature = "server")]`), not at the crate root.

# Game Flow

Games are created in **active** status immediately — there is no lobby phase to navigate through.

```
Create game → navigate directly to /game/:id/hider or /game/:id/seeker (based on chosen role)
Join game   → navigate directly to /game/:id/hider or /game/:id/seeker (based on chosen role)
```

- The lobby route (`/game/:id/lobby`) still exists but is not used by the normal flow.
- Joining an active game is allowed (players can join mid-game).
- A game is fully playable with only one team — no minimum player count.

# Common Dioxus 0.7 Gotchas

**`Navigator::push` return type**: Returns `Option<ExternalNavigationFailure>`, not `()`. Discard with `let _ = nav.push(...)` — otherwise closures passed to `EventHandler<()>` fail the `SpawnIfAsync` bound.

**`EventHandler<()>` closure type**: Explicit type annotation needed: `move |_: ()| { ... }` (not `move |_| ...`).

**Resource borrow lifetime**: `match &*resource.read() { ... }` — the temporary read guard must outlive the returned `Element`. Store in a binding first:
```rust
let guard = resource.read();
let x = match &*guard { ... };
x
```

**`Signal::write()` requires `mut`**: All signals that are ever written must be declared `let mut s = use_signal(...)`.

**Borrow conflict on toggle**: `signal.set(!*signal.read())` borrows twice. Split it:
```rust
let v = *signal.read();
signal.set(!v);
```

**PartialEq on all props**: Every struct used as a component prop must derive `PartialEq`. Add it to any model types used directly in props.

**rand 0.9 API changes**: `distributions` → `distr`; `DistString` → `SampleString`; `thread_rng()` → `rng()`; `SliceRandom::choose` moved to `IndexedRandom::choose`.

**axum 0.8 + dioxus-fullstack 0.7**: `dioxus-fullstack 0.7.9` depends on `axum = "0.8.4"` and `axum-core = "0.5.2"`. Using axum 0.7 causes `FromRequest` trait mismatch in `FullstackContext::extract()`.

**axum 0.8 path param syntax**: Use `{param}` not `:param` in `.route()` strings. Example: `.route("/api/ws/{game_id}", ...)`. The old colon syntax panics at runtime with "Path segments must not start with `:`".

**uuid on WASM**: Add `js` feature to the `uuid` workspace dependency so `v4` UUIDs work on `wasm32-unknown-unknown`: `uuid = { version = "1", features = ["v4", "serde", "js"] }`.
