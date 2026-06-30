use api::endpoints::game::CreateGameResponse;
use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn HostSetup() -> Element {
    let nav = use_navigator();
    rsx! {
        ui::host_setup::HostSetupForm {
            on_created: move |resp: CreateGameResponse| {
                let game_id = resp.game_id;
                nav.push(Route::GameView { game_id });
            },
        }
    }
}
