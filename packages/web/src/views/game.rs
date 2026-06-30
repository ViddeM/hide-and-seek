use dioxus::CapturedError;
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn GameView(game_id: Uuid) -> Element {
    let data = use_resource(move || async move {
        let game = api::endpoints::game::get_game(game_id).await?;
        let map = api::endpoints::maps::get_map(game.map_id).await?;
        Ok::<_, CapturedError>((game, map))
    });

    match &*data.read() {
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
        Some(Ok((game, map))) => rsx! {
            ui::game_view::GameView { game: game.clone(), map: map.clone() }
        },
    }
}
