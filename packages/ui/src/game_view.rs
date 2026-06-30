use api::{
    endpoints::{
        exclusion_zone::{AddZoneRequest, ExclusionZoneResponse},
        game::GameResponse,
        maps::MapDetailResponse,
    },
    types::{
        Point,
        area::{Area, Circle},
    },
};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn GameView(game: GameResponse, map: MapDetailResponse) -> Element {
    let mut zones: Signal<Vec<ExclusionZoneResponse>> = use_signal(Vec::new);
    let mut show_add = use_signal(|| false);

    // Load initial zones
    use_resource(move || async move {
        if let Ok(loaded) =
            api::endpoints::exclusion_zone::list_game_exclusion_zones(game.game_id).await
        {
            zones.write().clone_from(&loaded);
        }
    });

    // WebSocket for real-time zone updates
    use_effect(move || {
        let ws_js = format!(
            r#"(function(){{
                if(window._hideseekWs)return;
                var ws=new WebSocket('ws://'+location.host+'/api/ws/{}');
                ws.onmessage=function(e){{
                    var msg=JSON.parse(e.data);
                    if(msg.type==='ping'){{ws.send(JSON.stringify({{type:'pong'}}));}}
                }};
                window._hideseekWs=ws;
            }})();"#,
            game.game_id
        );
        let _ = document::eval(&ws_js);
    });

    rsx! {
        div { class: "seeker-view",
            div { class: "seeker-view__map",
                crate::MapView {
                    boundary: map.boundary.clone(),
                    zones: zones,
                }
            }

            div { class: "seeker-view__panel",
                h2 { "Exclusion Zones" }

                button {
                    class: "btn btn--primary",
                    onclick: move |_| { let v = *show_add.read(); show_add.set(!v); },
                    if *show_add.read() { "Cancel" } else { "+ Add Zone" }
                }

                if *show_add.read() {
                    AddZoneForm {
                        game_id: game.game_id,
                        on_added: move |zone: ExclusionZoneResponse| {
                            zones.write().push(zone);
                            show_add.set(false);
                        },
                    }
                }

                ul { class: "zone-list",
                    for zone in zones.read().clone() {
                        ZoneItem {
                            key: "{zone.id}",
                            zone: zone.clone(),
                            game_id: game.game_id,
                            on_removed: move |id: Uuid| {
                                zones.write().retain(|z| z.id != id);
                            },
                        }
                    }
                }

                crate::RadarExplorer {
                    game_id: game.game_id,
                    is_seeker: true,
                    on_zone_added: move |zone: ExclusionZoneResponse| {
                        zones.write().push(zone);
                    },
                }
            }
        }
    }
}

#[component]
fn AddZoneForm(game_id: Uuid, on_added: EventHandler<ExclusionZoneResponse>) -> Element {
    let mut lat = use_signal(String::new);
    let mut lng = use_signal(String::new);
    let mut radius = use_signal(|| "500".to_string());
    let mut label = use_signal(String::new);
    let mut exclude_outside = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        if *loading.read() {
            return;
        }

        let Ok(lat_v) = lat.read().trim().parse::<f64>() else {
            error.set(Some("Invalid latitude".to_string()));
            return;
        };
        let Ok(lng_v) = lng.read().trim().parse::<f64>() else {
            error.set(Some("Invalid longitude".to_string()));
            return;
        };
        let Ok(r_v) = radius.read().trim().parse::<u32>() else {
            error.set(Some("Invalid radius".to_string()));
            return;
        };

        loading.set(true);
        error.set(None);
        let req = AddZoneRequest {
            area: Area::Circle(Circle {
                center: Point {
                    lat: lat_v,
                    lng: lng_v,
                },
                radius: r_v as f64,
            }),
            exclude_outside: *exclude_outside.read(),
            label: {
                let l = label.read().trim().to_string();
                if l.is_empty() { None } else { Some(l) }
            },
        };

        spawn(async move {
            match api::endpoints::exclusion_zone::create_exclusion_zone(game_id, req).await {
                Ok(zone) => {
                    loading.set(false);
                    on_added.call(zone);
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
        form { class: "add-zone-form", onsubmit: submit,
            div { class: "form-row",
                label { r#for: "zone-lat", "Latitude" }
                input { id: "zone-lat", r#type: "text", placeholder: "51.5074",
                    oninput: move |e| lat.set(e.value()), value: lat.read().clone() }
            }
            div { class: "form-row",
                label { r#for: "zone-lng", "Longitude" }
                input { id: "zone-lng", r#type: "text", placeholder: "-0.1278",
                    oninput: move |e| lng.set(e.value()), value: lng.read().clone() }
            }
            div { class: "form-row",
                label { r#for: "zone-radius", "Radius (metres)" }
                input { id: "zone-radius", r#type: "number", min: "100", max: "50000",
                    oninput: move |e| radius.set(e.value()), value: radius.read().clone() }
            }
            div { class: "form-row",
                label { r#for: "zone-label", "Label (optional)" }
                input { id: "zone-label", r#type: "text", placeholder: "e.g. City centre",
                    oninput: move |e| label.set(e.value()), value: label.read().clone() }
            }
            div { class: "form-row form-row--check",
                label {
                    input {
                        r#type: "checkbox",
                        checked: *exclude_outside.read(),
                        onchange: move |e| exclude_outside.set(e.checked()),
                    }
                    "Hider is inside (exclude outside)"
                }
            }

            if let Some(msg) = error.read().as_ref() {
                p { class: "form-error", "{msg}" }
            }
            button { r#type: "submit", class: "btn btn--primary",
                disabled: *loading.read(),
                if *loading.read() { "Adding…" } else { "Add Zone" }
            }
        }
    }
}

#[component]
fn ZoneItem(zone: ExclusionZoneResponse, game_id: Uuid, on_removed: EventHandler<Uuid>) -> Element {
    let mut removing = use_signal(|| false);
    let zone_id = zone.id;

    let remove = move |_| {
        if *removing.read() {
            return;
        }
        removing.set(true);
        spawn(async move {
            if api::endpoints::exclusion_zone::remove_exclusion_zone(game_id, zone_id)
                .await
                .is_ok()
            {
                on_removed.call(zone_id);
            }
            removing.set(false);
        });
    };

    rsx! {
        li { class: "zone-item",
            div { class: "zone-item__info",
                if let Some(label) = &zone.label {
                    strong { "{label}" }
                    span { "  " }
                }
                span { "{zone.area.display()}" }
                if zone.exclude_outside {
                    span { class: "zone-item__tag", " · inside only" }
                }
            }
            button {
                class: "zone-item__remove",
                onclick: remove,
                disabled: *removing.read(),
                "×"
            }
        }
    }
}
