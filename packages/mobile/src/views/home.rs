use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        main {
            h1 { "Hide & Seek" }
            p { "Open the web app to play." }
        }
    }
}
