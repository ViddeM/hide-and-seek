use api::models::{DrawnCard, ExclusionZone, MapDetail, SessionInfo};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn HiderViewComponent(game_id: Uuid, session: SessionInfo, map: MapDetail) -> Element {
    let mut hand: Signal<Vec<DrawnCard>> = use_signal(Vec::new);
    let mut draw_error = use_signal(|| None::<String>);
    let mut drawing = use_signal(|| false);
    let mut zones: Signal<Vec<ExclusionZone>> = use_signal(Vec::new);

    // Load initial hand
    use_resource(move || async move {
        if let Ok(cards) = api::cards::get_hand(game_id).await {
            hand.write().clone_from(&cards);
        }
    });

    // Load zones so hiders can see what has been deduced
    use_resource(move || async move {
        if let Ok(loaded) = api::zones::list_exclusion_zones(game_id).await {
            zones.write().clone_from(&loaded);
        }
    });

    rsx! {
        div { class: "hider-view",
            div { class: "hider-view__map",
                crate::MapView {
                    boundary: map.boundary.clone(),
                    zones: zones,
                }
            }
            div { class: "hider-view__panel",
            section { class: "draw-section",
                h2 { "Draw Cards" }
                div { class: "draw-buttons",
                    button {
                        class: "btn btn--secondary",
                        onclick: move |_| {
                            if *drawing.read() { return; }
                            drawing.set(true);
                            draw_error.set(None);
                            spawn(async move {
                                match api::cards::draw_cards(game_id, 1).await {
                                    Ok(mut new_cards) => {
                                        drawing.set(false);
                                        hand.write().append(&mut new_cards);
                                    }
                                    Err(e) => {
                                        drawing.set(false);
                                        draw_error.set(Some(e.to_string()));
                                    }
                                }
                            });
                        },
                        disabled: *drawing.read(),
                        "Draw 1"
                    }
                    button {
                        class: "btn btn--secondary",
                        onclick: move |_| {
                            if *drawing.read() { return; }
                            drawing.set(true);
                            draw_error.set(None);
                            spawn(async move {
                                match api::cards::draw_cards(game_id, 2).await {
                                    Ok(mut new_cards) => {
                                        drawing.set(false);
                                        hand.write().append(&mut new_cards);
                                    }
                                    Err(e) => {
                                        drawing.set(false);
                                        draw_error.set(Some(e.to_string()));
                                    }
                                }
                            });
                        },
                        disabled: *drawing.read(),
                        "Draw 2"
                    }
                    button {
                        class: "btn btn--secondary",
                        onclick: move |_| {
                            if *drawing.read() { return; }
                            drawing.set(true);
                            draw_error.set(None);
                            spawn(async move {
                                match api::cards::draw_cards(game_id, 3).await {
                                    Ok(mut new_cards) => {
                                        drawing.set(false);
                                        hand.write().append(&mut new_cards);
                                    }
                                    Err(e) => {
                                        drawing.set(false);
                                        draw_error.set(Some(e.to_string()));
                                    }
                                }
                            });
                        },
                        disabled: *drawing.read(),
                        "Draw 3"
                    }
                }
                if let Some(msg) = draw_error.read().as_ref() {
                    p { class: "form-error", "{msg}" }
                }
            }

            section { class: "hand-section",
                h2 { "Your Hand ({hand.read().len()} cards)" }
                if hand.read().is_empty() {
                    p { class: "hand-empty", "No cards yet. Draw some!" }
                } else {
                    div { class: "card-grid",
                        for card in hand.read().clone() {
                            crate::CardDisplay {
                                key: "{card.id}",
                                card: card.clone(),
                                on_play: {
                                    let card_id = card.id;
                                    Some(EventHandler::new(move |_| {
                                        spawn(async move {
                                            if api::cards::play_card(game_id, card_id).await.is_ok() {
                                                hand.write().retain(|c| c.id != card_id);
                                            }
                                        });
                                    }))
                                },
                            }
                        }
                    }
                }
            }

            crate::RadarExplorer {
                game_id: game_id,
                is_seeker: false,
                on_zone_added: move |_: ExclusionZone| {},
            }
            } // hider-view__panel
        }
    }
}
