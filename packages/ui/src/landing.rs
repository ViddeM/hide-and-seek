use dioxus::prelude::*;

#[component]
pub fn Landing(on_join: EventHandler<String>, on_host: EventHandler<()>) -> Element {
    let mut code = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let val = code.read().trim().to_uppercase();
        if val.len() != 6 {
            error.set(Some("Enter the 6-character game code".to_string()));
            return;
        }
        error.set(None);
        on_join.call(val);
    };

    rsx! {
        main { class: "landing",
            h1 { class: "landing__title", "Hide & Seek" }
            p { class: "landing__subtitle", "Based on Jetlag: The Game" }

            form {
                class: "landing__form",
                onsubmit: submit,
                input {
                    class: "landing__code-input",
                    r#type: "text",
                    placeholder: "Game code (e.g. A3X7K2)",
                    maxlength: 6,
                    autocomplete: "off",
                    oninput: move |e| code.set(e.value().to_uppercase()),
                    value: code.read().clone(),
                }
                button { r#type: "submit", class: "btn btn--primary", "Join Game" }
            }

            if let Some(msg) = error.read().as_ref() {
                p { class: "landing__error", "{msg}" }
            }

            div { class: "landing__divider", "or" }

            button {
                class: "btn btn--secondary",
                onclick: move |_| on_host.call(()),
                "Host New Game"
            }
        }
    }
}
