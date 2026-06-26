use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn HostSetup() -> Element {
    let nav = use_navigator();
    rsx! {
        ui::host_setup::HostSetupForm {
            on_created: move |resp: api::models::CreateGameResponse| {
                let _ = nav.push(Route::Lobby { game_id: resp.game_id.to_string() });
            },
        }
    }
}
