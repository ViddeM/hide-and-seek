use dioxus::prelude::*;

#[component]
pub fn ErrorBanner(message: Option<String>) -> Element {
    let Some(msg) = message else {
        return rsx! {};
    };
    rsx! {
        div {
            class: "error-banner",
            role: "alert",
            span { "⚠ {msg}" }
        }
    }
}
