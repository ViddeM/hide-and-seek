use api::models::{SessionInfo, TeamInfo};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn LobbyView(
    game_id: Uuid,
    session: SessionInfo,
    on_started: EventHandler<()>,
) -> Element {
    let mut teams = use_resource(move || api::game::list_teams(game_id));
    let mut start_error = use_signal(|| None::<String>);
    let mut starting = use_signal(|| false);

    let start = move |_| {
        if *starting.read() { return; }
        starting.set(true);
        start_error.set(None);
        spawn(async move {
            match api::auth::start_game(game_id).await {
                Ok(()) => {
                    starting.set(false);
                    on_started.call(());
                }
                Err(e) => {
                    starting.set(false);
                    start_error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
        main { class: "lobby",
            h2 { "Waiting for players…" }

            button {
                class: "btn btn--ghost lobby__refresh",
                onclick: move |_| teams.restart(),
                "Refresh"
            }

            match &*teams.read() {
                None => rsx! { p { "Loading teams…" } },
                Some(Err(e)) => rsx! { p { class: "form-error", "Error: {e}" } },
                Some(Ok(team_list)) => rsx! {
                    ul { class: "lobby__teams",
                        for team in team_list {
                            TeamCard { key: "{team.id}", team: team.clone() }
                        }
                    }
                },
            }

            if let Some(msg) = start_error.read().as_ref() {
                p { class: "form-error", "{msg}" }
            }

            if session.is_host {
                button {
                    class: "btn btn--primary",
                    onclick: start,
                    disabled: *starting.read(),
                    if *starting.read() { "Starting…" } else { "Start Game" }
                }
            } else {
                p { class: "lobby__waiting", "Waiting for the host to start…" }
            }
        }
    }
}

#[component]
fn TeamCard(team: TeamInfo) -> Element {
    rsx! {
        li { class: "team-card",
            div { class: "team-card__header",
                strong { "{team.name}" }
                span { class: "team-card__role", " ({team.role:?})" }
            }
            ul { class: "team-card__players",
                for player in &team.players {
                    li {
                        "{player.display_name}"
                        if player.is_host {
                            span { class: "host-badge", " (host)" }
                        }
                    }
                }
            }
        }
    }
}
