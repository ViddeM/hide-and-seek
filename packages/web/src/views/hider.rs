use dioxus::prelude::*;

#[component]
pub fn HiderView(game_id: String) -> Element {
    let data = use_resource(move || {
        let _gid = game_id.clone();
        async move {
            api::auth::get_session()
                .await?
                .ok_or_else(|| ServerFnError::new("Not authenticated"))
        }
    });

    let read = data.read();
    let x = match &*read {
        None => rsx! {
            main { class: "loading", p { "Loading…" } }
        },
        Some(Err(e)) => {
            let msg = e.to_string();
            rsx! {
                main { class: "error-page",
                    p { class: "form-error", "{msg}" }
                    a { href: "/", "← Back to start" }
                }
            }
        }
        Some(Ok(session)) => rsx! {
            ui::hider_view::HiderViewComponent {
                game_id: session.game_id,
                session: session.clone(),
            }
        },
    };
    x
}
