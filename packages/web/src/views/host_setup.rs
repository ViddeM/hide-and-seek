use api::endpoints::game::CreateGameResponse;
use dioxus::prelude::*;

#[component]
pub fn HostSetup() -> Element {
    let nav = use_navigator();
    rsx! {
        ui::host_setup::HostSetupForm {
            on_created: move |resp: CreateGameResponse| {
                let game_id = resp.game_id.to_string();
                // nav.push(Route::GameView { game_id });
            },
        }
    }
}
