use api::endpoints::game::CreateGameResponse;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::Route;

#[component]
pub fn HostSetup() -> Element {
    let nav = use_navigator();
    rsx! {
        ui::host_setup::HostSetupForm {
            on_created: move |resp: CreateGameResponse| {
                let game_id = resp.game_id;
                let _ = nav.push(Route::GameView { game_id });
            },
            on_create_map: move |_: ()| {
                let _ = nav.push(Route::NewMap {});
            },
            on_continue_draft: move |id: Uuid| {
                // Resume at the boundary step — the user can navigate back to info from there
                let _ = nav.push(Route::EditMapBoundary { id });
            },
        }
    }
}
