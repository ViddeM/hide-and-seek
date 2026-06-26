use api::models::{AddZoneRequest, ExclusionZone};
use dioxus::prelude::*;
use uuid::Uuid;

/// Radar question explorer — shows a live preview circle on `window._hideseekMap`.
/// Seekers see apply buttons to commit the zone; hiders see a read-only preview.
#[component]
pub fn RadarExplorer(
    game_id: Uuid,
    is_seeker: bool,
    on_zone_added: EventHandler<ExclusionZone>,
) -> Element {
    let mut expanded = use_signal(|| false);
    let mut center: Signal<Option<[f64; 2]>> = use_signal(|| None);
    let mut radius_km = use_signal(|| 5.0f64);
    let mut show_yes = use_signal(|| false);
    let mut adding = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut picking_center = use_signal(|| false);

    // Redraw preview circle whenever center, radius, or yes/no changes
    use_effect(move || {
        let c = *center.read();
        let r_m = *radius_km.read() * 1000.0;
        let yes = *show_yes.read();
        let color = if yes { "#27ae60" } else { "#e74c3c" };
        let js = match c {
            Some([lat, lng]) => format!(
                r#"(function(){{
                    var m=window._hideseekMap;if(!m)return;
                    if(window._radarPreview){{m.removeLayer(window._radarPreview);}}
                    window._radarPreview=L.circle([{lat},{lng}],{{
                        radius:{r_m},color:'{color}',fillOpacity:0.18,dashArray:'8',weight:2
                    }}).addTo(m);
                }})()"#
            ),
            None => r#"(function(){
                var m=window._hideseekMap;
                if(m&&window._radarPreview){m.removeLayer(window._radarPreview);window._radarPreview=null;}
            })()"#
            .to_string(),
        };
        let _ = document::eval(&js);
    });

    let pick_center = move |_| {
        if *picking_center.read() {
            picking_center.set(false);
            return;
        }
        picking_center.set(true);
        spawn(async move {
            let mut eval = document::eval(
                r#"(async function() {
                    var tries = 0;
                    while (!window._hideseekMap && tries++ < 100) {
                        await new Promise(r => setTimeout(r, 100));
                    }
                    if (!window._hideseekMap) return;
                    window._hideseekMap.once('click', function(e) {
                        dioxus.send([e.latlng.lat, e.latlng.lng]);
                    });
                })()"#,
            );
            if let Ok(pt) = eval.recv::<[f64; 2]>().await {
                if *picking_center.read() {
                    center.set(Some(pt));
                    picking_center.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "questions-panel",
            button {
                class: "btn btn--ghost questions-panel__toggle",
                r#type: "button",
                onclick: move |_| { let v = *expanded.read(); expanded.set(!v); },
                if *expanded.read() { "Questions ▲" } else { "Questions ▼" }
            }

            if *expanded.read() {
                div { class: "radar-explorer",
                    h3 { class: "radar-explorer__title", "Radar" }
                    p { class: "radar-explorer__desc",
                        "\"Are you within X km of [location]?\""
                    }

                    div { class: "radar-explorer__row",
                        span { class: "radar-explorer__coords",
                            if let Some([lat, lng]) = *center.read() {
                                "{lat:.4}°, {lng:.4}°"
                            } else {
                                "No center set"
                            }
                        }
                        button {
                            class: if *picking_center.read() {
                                "btn btn--primary btn--sm"
                            } else {
                                "btn btn--ghost btn--sm"
                            },
                            r#type: "button",
                            onclick: pick_center,
                            if *picking_center.read() { "Click map to set…" } else { "Set center" }
                        }
                    }

                    div { class: "radar-explorer__row",
                        label { class: "radar-explorer__label", "Radius" }
                        input {
                            r#type: "range",
                            min: "1",
                            max: "50",
                            step: "0.5",
                            value: "{radius_km.read()}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<f64>() {
                                    radius_km.set(v);
                                }
                            },
                        }
                        span { class: "radar-explorer__radius-val", "{radius_km.read():.1} km" }
                    }

                    div { class: "radar-explorer__toggle",
                        button {
                            class: if !*show_yes.read() {
                                "btn btn--sm radar-toggle--no radar-toggle--active"
                            } else {
                                "btn btn--sm radar-toggle--no"
                            },
                            r#type: "button",
                            onclick: move |_| show_yes.set(false),
                            "No"
                        }
                        button {
                            class: if *show_yes.read() {
                                "btn btn--sm radar-toggle--yes radar-toggle--active"
                            } else {
                                "btn btn--sm radar-toggle--yes"
                            },
                            r#type: "button",
                            onclick: move |_| show_yes.set(true),
                            "Yes"
                        }
                    }

                    p { class: "radar-explorer__impact",
                        if *show_yes.read() {
                            "Yes → hider must be within this circle."
                        } else {
                            "No → this area is eliminated."
                        }
                    }

                    if is_seeker && center.read().is_some() {
                        div { class: "radar-explorer__apply",
                            button {
                                class: "btn btn--sm btn--danger",
                                r#type: "button",
                                disabled: *adding.read(),
                                onclick: move |_| {
                                    if *adding.read() { return; }
                                    let Some([lat, lng]) = *center.read() else { return; };
                                    let r_m = (*radius_km.read() * 1000.0) as u32;
                                    adding.set(true);
                                    error.set(None);
                                    let req = AddZoneRequest {
                                        center_lat: lat,
                                        center_lng: lng,
                                        radius_m: r_m,
                                        exclude_outside: true,
                                        label: Some("Radar: No".to_string()),
                                        question_id: None,
                                    };
                                    spawn(async move {
                                        match api::zones::add_exclusion_zone(game_id, req).await {
                                            Ok(zone) => {
                                                adding.set(false);
                                                center.set(None);
                                                on_zone_added.call(zone);
                                            }
                                            Err(e) => {
                                                adding.set(false);
                                                error.set(Some(e.to_string()));
                                            }
                                        }
                                    });
                                },
                                "Apply No (exclude)"
                            }
                            button {
                                class: "btn btn--sm btn--success",
                                r#type: "button",
                                disabled: *adding.read(),
                                onclick: move |_| {
                                    if *adding.read() { return; }
                                    let Some([lat, lng]) = *center.read() else { return; };
                                    let r_m = (*radius_km.read() * 1000.0) as u32;
                                    adding.set(true);
                                    error.set(None);
                                    let req = AddZoneRequest {
                                        center_lat: lat,
                                        center_lng: lng,
                                        radius_m: r_m,
                                        exclude_outside: false,
                                        label: Some("Radar: Yes".to_string()),
                                        question_id: None,
                                    };
                                    spawn(async move {
                                        match api::zones::add_exclusion_zone(game_id, req).await {
                                            Ok(zone) => {
                                                adding.set(false);
                                                center.set(None);
                                                on_zone_added.call(zone);
                                            }
                                            Err(e) => {
                                                adding.set(false);
                                                error.set(Some(e.to_string()));
                                            }
                                        }
                                    });
                                },
                                "Apply Yes (include)"
                            }
                        }
                    }

                    if let Some(msg) = error.read().as_ref() {
                        p { class: "form-error", "{msg}" }
                    }
                }
            }
        }
    }
}
