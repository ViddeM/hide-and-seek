use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn JoinGame() -> Element {
    let nav = use_navigator();
    let mut initial_code: Signal<Option<String>> = use_signal(|| None);

    // Read pre-filled code from sessionStorage (no-op during SSR)
    use_effect(move || {
        spawn(async move {
            let mut eval = document::eval("sessionStorage.getItem('joinCode') || ''");
            if let Ok(code) = eval.recv::<String>().await {
                let code = code.trim().to_uppercase();
                if code.len() == 6 {
                    initial_code.set(Some(code));
                    let _ = document::eval("sessionStorage.removeItem('joinCode')");
                }
            }
        });
    });

    rsx! {
        ui::join::JoinForm {
            initial_code: initial_code.read().clone(),
            on_joined: move |resp: api::models::JoinGameResponse| {
                let game_id = resp.game_id.to_string();
                let route = if resp.role == api::models::TeamRole::Seeker {
                    Route::SeekerView { game_id }
                } else {
                    Route::HiderView { game_id }
                };
                let _ = nav.push(route);
            },
        }
    }
}
