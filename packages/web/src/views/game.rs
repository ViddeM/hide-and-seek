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

    let (game, map) = (*data.suspend()?.read()).clone()?;

    rsx! {
        ui::game_view::GameView { game, map }
    }
}
