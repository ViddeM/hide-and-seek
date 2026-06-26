use dioxus::prelude::*;

#[component]
pub fn HiderView(game_id: String) -> Element {
    let data = use_resource(move || {
        let _gid = game_id.clone();
        async move {
            let session = api::auth::get_session()
                .await?
                .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
            let state = api::game::get_game_state(session.game_id).await?;
            let map = api::maps::get_map(state.map_id).await?;
            Ok::<_, ServerFnError>((session, map))
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
        Some(Ok((session, map))) => rsx! {
            ui::hider_view::HiderViewComponent {
                game_id: session.game_id,
                session: session.clone(),
                map: map.clone(),
            }
        },
    };
    x
}
