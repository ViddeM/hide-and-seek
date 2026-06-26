use api::models::{ExclusionZone, MapDetail, SessionInfo, TeamInfo};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn HostViewComponent(
    game_id: Uuid,
    session: SessionInfo,
    map: MapDetail,
) -> Element {
    let teams = use_resource(move || api::game::list_teams(game_id));
    let zones_res = use_resource(move || api::zones::list_exclusion_zones(game_id));
    let mut turn_error = use_signal(|| None::<String>);

    let start_turn = move |hiding_team_id: Uuid| {
        turn_error.set(None);
        spawn(async move {
            if let Err(e) = api::game::start_turn(game_id, hiding_team_id).await {
                turn_error.set(Some(e.to_string()));
            }
        });
    };

    let empty_zones: Vec<ExclusionZone> = Vec::new();
    let zone_list = match &*zones_res.read() {
        Some(Ok(z)) => z.clone(),
        _ => empty_zones,
    };

    rsx! {
        div { class: "host-view",
            div { class: "host-view__map",
                crate::MapView {
                    center_lat: (map.bounds.sw_lat + map.bounds.ne_lat) / 2.0,
                    center_lng: (map.bounds.sw_lng + map.bounds.ne_lng) / 2.0,
                    sw_lat: map.bounds.sw_lat,
                    sw_lng: map.bounds.sw_lng,
                    ne_lat: map.bounds.ne_lat,
                    ne_lng: map.bounds.ne_lng,
                    zones: Signal::new(zone_list),
                }
            }

            div { class: "host-view__panel",
                h2 { "Teams" }

                match &*teams.read() {
                    None => rsx! { p { "Loading…" } },
                    Some(Err(e)) => rsx! { p { class: "form-error", "Error: {e}" } },
                    Some(Ok(team_list)) => rsx! {
                        div { class: "host-teams",
                            for team in team_list {
                                HostTeamCard {
                                    key: "{team.id}",
                                    team: team.clone(),
                                    on_start_turn: start_turn,
                                }
                            }
                        }
                    },
                }

                if let Some(msg) = turn_error.read().as_ref() {
                    p { class: "form-error", "{msg}" }
                }
            }
        }
    }
}

#[component]
fn HostTeamCard(team: TeamInfo, on_start_turn: EventHandler<Uuid>) -> Element {
    let team_id = team.id;
    rsx! {
        div { class: "host-team-card",
            div { class: "host-team-card__header",
                strong { "{team.name}" }
                span { " ({team.role:?})" }
            }
            ul {
                for p in &team.players {
                    li {
                        "{p.display_name}"
                        if p.is_host { " ★" }
                    }
                }
            }
            button {
                class: "btn btn--secondary",
                onclick: move |_| on_start_turn.call(team_id),
                "Start turn (hiding)"
            }
        }
    }
}
