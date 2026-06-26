use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Lobby(game_id: String) -> Element {
    let nav = use_navigator();
    let gid = game_id.clone();
    let session = use_resource(move || api::auth::get_session());

    let read = session.read();
    let x = match &*read {
        None => rsx! {
            main { class: "loading", p { "Loading session…" } }
        },
        Some(Err(e)) => {
            let msg = e.to_string();
            rsx! {
                main { class: "error-page",
                    p { class: "form-error", "Session error: {msg}" }
                    a { href: "/", "← Back to start" }
                }
            }
        }
        Some(Ok(None)) => rsx! {
            main { class: "error-page",
                p { class: "form-error", "Not logged in." }
                a { href: "/", "← Back to start" }
            }
        },
        Some(Ok(Some(s))) => {
            let session = s.clone();
            let game_id_owned = gid.clone();
            rsx! {
                ui::lobby::LobbyView {
                    game_id: session.game_id,
                    session: session.clone(),
                    on_started: move |_: ()| {
                        let route = if session.is_host {
                            Route::HostView { game_id: game_id_owned.clone() }
                        } else if session.role == api::models::TeamRole::Seeker {
                            Route::SeekerView { game_id: game_id_owned.clone() }
                        } else {
                            Route::HiderView { game_id: game_id_owned.clone() }
                        };
                        let _ = nav.push(route);
                    },
                }
            }
        }
    };
    x
}
