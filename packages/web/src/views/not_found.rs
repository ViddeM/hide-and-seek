use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        main { class: "not-found",
            h1 { "404" }
            p { "Page not found: /{segments.join(\"/\")}" }
            a { href: "/", class: "btn btn--secondary", "← Back to start" }
        }
    }
}
