use api::endpoints::game::CreateGameResponse;
use dioxus::prelude::*;

#[component]
pub fn HostSetup() -> Element {
    let nav = use_navigator();
    rsx! {
        ui::host_setup::HostSetupForm {
            on_created: move |resp: CreateGameResponse| {
                let game_id = resp.game_id.to_string();
            },
            // on_created: move |resp: api::models::CreateGameResponse| {
            //     let game_id = resp.game_id.to_string();
            //     // let route = if resp.role == api::models::TeamRole::Seeker {
            //     //     // Route::SeekerView { game_id }
            //     // } else {
            //     //     // Route::HiderView { game_id }
            //     // };
            //     let _ = nav.push(route);
            // },
        }
    }
}
