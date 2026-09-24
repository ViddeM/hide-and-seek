use api::{
    endpoints::maps::{CreateMapRequest, MapSummary, PreviewTransitRequest},
    types::{
        Point, area::Polygon, map_size::MapSize,
        transit::{TransitRoute, TransitRouteType},
    },
};
use dioxus::prelude::*;

use crate::BoundaryMapEditor;

#[derive(Clone, Copy, PartialEq)]
enum WizardStep {
    Boundary,
    Transit,
}

fn route_type_label(rt: TransitRouteType) -> &'static str {
    match rt {
        TransitRouteType::Bus => "Bus",
        TransitRouteType::Tram => "Tram",
        TransitRouteType::Subway => "Subway",
        TransitRouteType::Train => "Train",
        TransitRouteType::Ferry => "Ferry",
        TransitRouteType::LightRail => "Light Rail",
        TransitRouteType::Monorail => "Monorail",
        TransitRouteType::Trolleybus => "Trolleybus",
        TransitRouteType::Funicular => "Funicular",
    }
}

fn route_type_color(rt: TransitRouteType) -> &'static str {
    match rt {
        TransitRouteType::Bus => "#1a73e8",
        TransitRouteType::Tram => "#e8a81a",
        TransitRouteType::Subway => "#dc3545",
        TransitRouteType::Train => "#198754",
        TransitRouteType::Ferry => "#0dcaf0",
        TransitRouteType::LightRail => "#6f42c1",
        TransitRouteType::Monorail => "#fd7e14",
        TransitRouteType::Trolleybus => "#20c997",
        TransitRouteType::Funicular => "#d63384",
    }
}

fn effective_color(route: &TransitRoute) -> String {
    route.color.clone().unwrap_or_else(|| route_type_color(route.route_type).to_string())
}

const MODE_ORDER: &[TransitRouteType] = &[
    TransitRouteType::Train,
    TransitRouteType::Subway,
    TransitRouteType::Tram,
    TransitRouteType::LightRail,
    TransitRouteType::Monorail,
    TransitRouteType::Trolleybus,
    TransitRouteType::Bus,
    TransitRouteType::Ferry,
    TransitRouteType::Funicular,
];

#[component]
pub fn CreateMapWizard(on_created: EventHandler<MapSummary>) -> Element {
    let mut step = use_signal(|| WizardStep::Boundary);
    let mut map_name = use_signal(String::new);
    let mut map_size = use_signal(|| MapSize::Medium);
    let mut boundary: Signal<Vec<Point>> = use_signal(Vec::new);

    let mut transit_routes: Signal<Vec<TransitRoute>> = use_signal(Vec::new);
    let mut route_selected: Signal<Vec<bool>> = use_signal(Vec::new);
    let mut transit_loading = use_signal(|| false);
    let mut transit_error = use_signal(|| None::<String>);
    let mut save_error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);

    // Sync selected transit routes to the boundary editor map preview
    use_effect(move || {
        let routes = transit_routes.read().clone();
        let selected = route_selected.read().clone();
        let is_transit = *step.read() == WizardStep::Transit;

        if !is_transit {
            let _ = document::eval("if(window._bndEditor&&window._bndTransitLayers){window._bndTransitLayers.forEach(function(l){try{window._bndEditor.removeLayer(l);}catch(e){}});window._bndTransitLayers=[];}");
            return;
        }

        let selected_routes: Vec<&TransitRoute> = routes
            .iter()
            .zip(selected.iter())
            .filter(|(_, s)| **s)
            .map(|(r, _)| r)
            .collect();

        let routes_json = serde_json::to_string(&selected_routes).unwrap_or_default();
        let js = format!(
            r#"(function syncTransit(){{
                var m=window._bndEditor;
                if(!m){{setTimeout(syncTransit,200);return;}}
                if(window._bndTransitLayers){{window._bndTransitLayers.forEach(function(l){{try{{m.removeLayer(l);}}catch(e){{}}}});}}
                window._bndTransitLayers=[];
                var colors={{bus:'#1a73e8',tram:'#e8a81a',subway:'#dc3545',train:'#198754',ferry:'#0dcaf0',light_rail:'#6f42c1',monorail:'#fd7e14',trolleybus:'#20c997',funicular:'#d63384'}};
                var routes={routes_json};
                routes.forEach(function(r){{
                    var color=r.color||colors[r.route_type]||'#888';
                    (r.waypoints||[]).forEach(function(seg){{
                        if(seg.length>1){{
                            var pts=seg.map(function(p){{return[p.lat,p.lng];}});
                            var line=L.polyline(pts,{{color:color,weight:3,opacity:0.85,interactive:false}}).addTo(m);
                            line.bindTooltip(r.name);
                            window._bndTransitLayers.push(line);
                        }}
                    }});
                    (r.stops||[]).forEach(function(s){{
                        var dot=L.circleMarker([s.lat,s.lng],{{radius:5,color:'#fff',weight:1.5,fillColor:color,fillOpacity:1}}).addTo(m);
                        dot.bindTooltip(s.name);
                        window._bndTransitLayers.push(dot);
                    }});
                }});
            }})();"#
        );
        let _ = document::eval(&js);
    });

    let fetch_transit = move |_| {
        if *transit_loading.read() {
            return;
        }
        transit_loading.set(true);
        transit_error.set(None);
        let size = *map_size.read();
        let pts = boundary.read().clone();
        spawn(async move {
            match api::endpoints::maps::preview_transit(PreviewTransitRequest {
                size,
                bounds: Polygon { vertices: pts },
            })
            .await
            {
                Ok(data) => {
                    let n = data.routes.len();
                    transit_routes.set(data.routes);
                    route_selected.set(vec![true; n]);
                    transit_loading.set(false);
                    step.set(WizardStep::Transit);
                }
                Err(e) => {
                    transit_loading.set(false);
                    transit_error.set(Some(e.to_string()));
                }
            }
        });
    };

    let can_fetch = boundary.read().len() >= 3 && !map_name.read().trim().is_empty();
    let is_transit = *step.read() == WizardStep::Transit;

    rsx! {
        main { class: "wizard-page",

            // ── Step indicator ───────────────────────────────────────
            div { class: "wizard-steps",
                div {
                    class: if is_transit { "wizard-step wizard-step--done" } else { "wizard-step wizard-step--active" },
                    span { class: "wizard-step__num", "1" }
                    span { class: "wizard-step__label", "Boundary" }
                }
                div { class: "wizard-steps__connector" }
                div {
                    class: if is_transit { "wizard-step wizard-step--active" } else { "wizard-step wizard-step--upcoming" },
                    span { class: "wizard-step__num", "2" }
                    span { class: "wizard-step__label", "Transit" }
                }
            }

            // ── Main layout: left panel + map ────────────────────────
            div { class: "wizard-body",

                // Left panel
                div { class: "wizard-left",

                    // Step 1 content
                    div { style: if is_transit { "display:none" } else { "" },
                        div { class: "wizard-field",
                            label { r#for: "wiz-map-name", "Map Name" }
                            input {
                                id: "wiz-map-name",
                                r#type: "text",
                                placeholder: "e.g. City Centre",
                                oninput: move |e| map_name.set(e.value()),
                                value: map_name.read().clone(),
                            }
                        }

                        div { class: "wizard-field",
                            label { r#for: "wiz-map-size", "Size" }
                            select {
                                id: "wiz-map-size",
                                onchange: move |e| {
                                    map_size.set(match e.value().as_str() {
                                        "small" => MapSize::Small,
                                        "large" => MapSize::Large,
                                        _ => MapSize::Medium,
                                    });
                                },
                                option { value: "small", "Small" }
                                option { value: "medium", selected: true, "Medium" }
                                option { value: "large", "Large" }
                            }
                        }

                        // Waypoint summary
                        {
                            let pts = boundary.read().clone();
                            let n = pts.len();
                            rsx! {
                                p { class: "wizard-hint",
                                    if n >= 3 {
                                        "{n} waypoint(s) placed — polygon ready"
                                    } else if n > 0 {
                                        "{n} waypoint(s) — need at least 3"
                                    } else {
                                        "Click on the map to place boundary waypoints"
                                    }
                                }
                                if !pts.is_empty() {
                                    ul { class: "waypoints-list wizard-waypoints",
                                        for (i, pt) in pts.iter().enumerate() {
                                            {
                                                let lat = pt.lat;
                                                let lng = pt.lng;
                                                rsx! {
                                                    li { class: "waypoint-item", key: "{i}",
                                                        span { class: "waypoint-item__num", "{i + 1}" }
                                                        span { class: "waypoint-item__coords", "{lat:.4}, {lng:.4}" }
                                                        button {
                                                            r#type: "button",
                                                            class: "waypoint-item__remove",
                                                            onclick: move |_| { boundary.write().remove(i); },
                                                            "×"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Error + fallback actions
                        if let Some(msg) = transit_error.read().clone() {
                            p { class: "form-error wizard-error", "{msg}" }
                            div { class: "wizard-actions",
                                button {
                                    r#type: "button",
                                    class: "btn btn--ghost",
                                    disabled: *saving.read(),
                                    onclick: move |_| {
                                        if *saving.read() { return; }
                                        let name = map_name.read().trim().to_string();
                                        if name.is_empty() { save_error.set(Some("Map name is required".into())); return; }
                                        let pts = boundary.read().clone();
                                        if pts.len() < 3 { save_error.set(Some("At least 3 boundary points required".into())); return; }
                                        save_error.set(None);
                                        saving.set(true);
                                        let size = *map_size.read();
                                        spawn(async move {
                                            match api::endpoints::maps::create_map(CreateMapRequest {
                                                name, size,
                                                bounds: Polygon { vertices: pts },
                                                transit_routes: vec![],
                                            }).await {
                                                Ok(map) => { saving.set(false); on_created.call(map); }
                                                Err(e) => { saving.set(false); save_error.set(Some(e.to_string())); }
                                            }
                                        });
                                    },
                                    if *saving.read() { "Saving…" } else { "Save without transit" }
                                }
                                button {
                                    r#type: "button",
                                    class: "btn btn--secondary",
                                    disabled: !can_fetch || *transit_loading.read(),
                                    onclick: fetch_transit,
                                    if *transit_loading.read() { "Fetching…" } else { "Retry" }
                                }
                            }
                        } else {
                            div { class: "wizard-actions",
                                button {
                                    r#type: "button",
                                    class: "btn btn--primary",
                                    disabled: !can_fetch || *transit_loading.read(),
                                    onclick: fetch_transit,
                                    if *transit_loading.read() { "Fetching transit…" } else { "Next: Fetch Transit →" }
                                }
                            }
                        }
                    }

                    // Step 2 content: transit selector
                    if is_transit {
                        div { class: "transit-panel",
                            {
                                let routes_snap = transit_routes.read().clone();
                                let sel_snap = route_selected.read().clone();
                                let total = routes_snap.len();
                                let n_sel = sel_snap.iter().filter(|&&s| s).count();

                                rsx! {
                                    p { class: "transit-panel__summary",
                                        "{n_sel} of {total} route(s) selected"
                                    }

                                    div { class: "transit-mode-list",
                                        for &mode in MODE_ORDER.iter() {
                                            {
                                                let mode_data: Vec<(usize, String, Option<String>, usize, String)> =
                                                    routes_snap.iter().enumerate()
                                                        .filter(|(_, r)| r.route_type == mode)
                                                        .map(|(i, r)| (i, r.name.clone(), r.long_name.clone(), r.stops.len(), effective_color(r)))
                                                        .collect();

                                                if mode_data.is_empty() {
                                                    rsx! {}
                                                } else {
                                                    let mode_indices: Vec<usize> = mode_data.iter().map(|(i, ..)| *i).collect();
                                                    let any_sel = mode_indices.iter().any(|&i| sel_snap.get(i).copied().unwrap_or(true));
                                                    let mode_label = route_type_label(mode);
                                                    let mode_color = route_type_color(mode);
                                                    let route_count = mode_data.len();
                                                    let toggle_indices = mode_indices.clone();

                                                    rsx! {
                                                        div { class: "transit-mode-group",
                                                            label { class: "transit-mode-header",
                                                                input {
                                                                    r#type: "checkbox",
                                                                    checked: any_sel,
                                                                    onchange: move |e| {
                                                                        let checked = e.checked();
                                                                        let mut sel = route_selected.write();
                                                                        for &idx in &toggle_indices {
                                                                            if let Some(v) = sel.get_mut(idx) { *v = checked; }
                                                                        }
                                                                    },
                                                                }
                                                                span {
                                                                    class: "mode-swatch",
                                                                    style: "background:{mode_color}",
                                                                }
                                                                span { class: "transit-mode-header__label", "{mode_label}" }
                                                                span { class: "transit-mode-header__count", "({route_count})" }
                                                            }

                                                            ul { class: "transit-route-list",
                                                                for (global_idx, route_name, long_name, stop_count, color) in mode_data.into_iter() {
                                                                    {
                                                                        let is_checked = sel_snap.get(global_idx).copied().unwrap_or(true);
                                                                        rsx! {
                                                                            li { class: "transit-route-item",
                                                                                label { class: "transit-route-item__label",
                                                                                    input {
                                                                                        r#type: "checkbox",
                                                                                        checked: is_checked,
                                                                                        onchange: move |e| {
                                                                                            let mut sel = route_selected.write();
                                                                                            if let Some(v) = sel.get_mut(global_idx) {
                                                                                                *v = e.checked();
                                                                                            }
                                                                                        },
                                                                                    }
                                                                                    span {
                                                                                        class: "route-swatch",
                                                                                        style: "background:{color}",
                                                                                    }
                                                                                    div { class: "transit-route-item__info",
                                                                                        strong { "{route_name}" }
                                                                                        if let Some(ln) = long_name {
                                                                                            span { class: "transit-route-item__longname", " – {ln}" }
                                                                                        }
                                                                                        span { class: "transit-route-item__meta",
                                                                                            " · {stop_count} stop(s)"
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if let Some(msg) = save_error.read().clone() {
                                        p { class: "form-error wizard-error", "{msg}" }
                                    }

                                    div { class: "wizard-actions",
                                        button {
                                            r#type: "button",
                                            class: "btn btn--ghost",
                                            onclick: move |_| {
                                                step.set(WizardStep::Boundary);
                                                transit_error.set(None);
                                                save_error.set(None);
                                            },
                                            "← Back"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "btn btn--primary",
                                            disabled: *saving.read(),
                                            onclick: move |_| {
                                                if *saving.read() { return; }
                                                let name = map_name.read().trim().to_string();
                                                if name.is_empty() { save_error.set(Some("Map name is required".into())); return; }
                                                let pts = boundary.read().clone();
                                                if pts.len() < 3 { save_error.set(Some("At least 3 boundary points required".into())); return; }
                                                let routes: Vec<TransitRoute> = transit_routes.read().clone()
                                                    .into_iter()
                                                    .zip(route_selected.read().clone().into_iter())
                                                    .filter(|(_, sel)| *sel)
                                                    .map(|(r, _)| r)
                                                    .collect();
                                                save_error.set(None);
                                                saving.set(true);
                                                let size = *map_size.read();
                                                spawn(async move {
                                                    match api::endpoints::maps::create_map(CreateMapRequest {
                                                        name, size,
                                                        bounds: Polygon { vertices: pts },
                                                        transit_routes: routes,
                                                    }).await {
                                                        Ok(map) => { saving.set(false); on_created.call(map); }
                                                        Err(e) => { saving.set(false); save_error.set(Some(e.to_string())); }
                                                    }
                                                });
                                            },
                                            if *saving.read() { "Saving…" } else { "Save Map" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Right panel: always-mounted map ──────────────────
                div { class: "wizard-right",
                    BoundaryMapEditor {
                        boundary,
                        show_waypoints: false,
                    }
                }
            }
        }
    }
}
