use api::{
    endpoints::maps::{
        PreviewTransitRequest, SaveTransitRequest, get_map, get_transit, preview_transit,
        save_map_transit,
    },
    types::{Point, area::Polygon, transit::{TransitRoute, TransitRouteType}},
};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::Route;
use super::WizardSteps;

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
    route
        .color
        .clone()
        .unwrap_or_else(|| route_type_color(route.route_type).to_string())
}

/// Step 3: fetch / select transit routes.
#[component]
pub fn EditMapTransit(id: Uuid) -> Element {
    let nav = use_navigator();

    let map_res = use_resource(move || async move { get_map(id).await });
    let transit_res = use_resource(move || async move { get_transit(id).await });

    // All signals unconditional — populated once resources resolve
    let mut boundary: Signal<Vec<Point>> = use_signal(Vec::new);
    let mut transit_routes: Signal<Vec<TransitRoute>> = use_signal(Vec::new);
    let mut route_selected: Signal<Vec<bool>> = use_signal(Vec::new);
    let mut transit_loading = use_signal(|| false);
    let mut transit_error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);
    let mut save_error = use_signal(|| None::<String>);
    let mut map_size = use_signal(|| api::types::map_size::MapSize::Medium);
    let mut initialized = use_signal(|| false);

    // Populate from DB once both resources are ready
    use_effect(move || {
        if *initialized.read() {
            return;
        }
        let map_guard = map_res.read();
        let transit_guard = transit_res.read();
        if let (Some(Ok(map)), Some(Ok(transit))) = (&*map_guard, &*transit_guard) {
            *boundary.write() = map.boundary.vertices.clone();
            map_size.set(map.size);
            let n = transit.routes.len();
            transit_routes.set(transit.routes.clone());
            route_selected.set(vec![true; n]);
            initialized.set(true);
        }
    });

    // Sync selected routes to the boundary-editor map preview.
    // Split into two evals: one to set data on a JS global, one to run a static
    // render function. This avoids building one huge concatenated string in WASM.
    use_effect(move || {
        let routes = transit_routes.read().clone();
        let selected = route_selected.read().clone();
        let selected_routes: Vec<&TransitRoute> = routes
            .iter()
            .zip(selected.iter())
            .filter(|(_, s)| **s)
            .map(|(r, _)| r)
            .collect();

        let routes_json = serde_json::to_string(&selected_routes).unwrap_or_default();

        // Step 1: push data into a JS global (small eval = just the JSON)
        let set_data = format!("window._transitRouteData={routes_json};");
        let _ = document::eval(&set_data);

        // Step 2: run a static render function that reads from the global.
        // Lines are drawn first, then deduplicated stop markers are added on top
        // via markerPane so no line ever obscures a station.
        let _ = document::eval(
            r#"(function syncTransit(){
                var m=window._bndEditor;
                if(!m){setTimeout(syncTransit,200);return;}
                if(window._bndTransitLayers){window._bndTransitLayers.forEach(function(l){try{m.removeLayer(l);}catch(e){}});}
                window._bndTransitLayers=[];
                var colors={bus:'#1a73e8',tram:'#e8a81a',subway:'#dc3545',train:'#198754',ferry:'#0dcaf0',light_rail:'#6f42c1',monorail:'#fd7e14',trolleybus:'#20c997',funicular:'#d63384'};
                var routes=window._transitRouteData||[];

                // Draw route polylines first (overlayPane, below markers)
                routes.forEach(function(r){
                    var color=r.color||colors[r.route_type]||'#888';
                    (r.waypoints||[]).forEach(function(seg){
                        if(seg.length>1){
                            var pts=seg.map(function(p){return[p.lat,p.lng];});
                            var line=L.polyline(pts,{color:color,weight:3,opacity:0.85,interactive:false}).addTo(m);
                            line.bindTooltip(r.name);
                            window._bndTransitLayers.push(line);
                        }
                    });
                });

                // Collect stops, deduplicate by name, average positions across platforms
                var stopMap={};
                routes.forEach(function(r){
                    var color=r.color||colors[r.route_type]||'#888';
                    (r.stops||[]).forEach(function(s){
                        var key=s.name.toLowerCase().trim();
                        if(!stopMap[key]){
                            stopMap[key]={latSum:s.lat,lngSum:s.lng,count:1,name:s.name,color:color};
                        } else {
                            stopMap[key].latSum+=s.lat;
                            stopMap[key].lngSum+=s.lng;
                            stopMap[key].count+=1;
                        }
                    });
                });

                // Draw deduplicated stops on markerPane (always above polylines)
                Object.keys(stopMap).forEach(function(k){
                    var s=stopMap[k];
                    var lat=s.latSum/s.count;
                    var lng=s.lngSum/s.count;
                    var dot=L.circleMarker([lat,lng],{
                        pane:'markerPane',radius:5,
                        color:'#fff',weight:1.5,
                        fillColor:s.color,fillOpacity:1
                    }).addTo(m);
                    dot.bindTooltip(s.name,{direction:'top',offset:[0,-7],className:'transit-stop-label'});
                    window._bndTransitLayers.push(dot);
                });
            })();"#,
        );
    });

    let fetch_transit_fn = move |_| {
        if *transit_loading.read() {
            return;
        }
        transit_loading.set(true);
        transit_error.set(None);
        let pts = boundary.read().clone();
        let size = *map_size.read();
        spawn(async move {
            match preview_transit(PreviewTransitRequest {
                size,
                bounds: Polygon { vertices: pts },
            })
            .await
            {
                Ok(data) => {
                    let routes = data.routes;
                    let n = routes.len();
                    // Persist immediately so a page reload restores the fetched routes.
                    // Re-fetching replaces whatever was stored before.
                    if let Err(e) = save_map_transit(id, SaveTransitRequest { routes: routes.clone() }).await {
                        transit_loading.set(false);
                        transit_error.set(Some(format!("Fetched {n} route(s) but could not save: {e}")));
                        return;
                    }
                    transit_routes.set(routes);
                    route_selected.set(vec![true; n]);
                    transit_loading.set(false);
                }
                Err(e) => {
                    transit_loading.set(false);
                    transit_error.set(Some(e.to_string()));
                }
            }
        });
    };

    let save_and_next = move |_| {
        if *saving.read() {
            return;
        }
        let routes: Vec<TransitRoute> = transit_routes
            .read()
            .clone()
            .into_iter()
            .zip(route_selected.read().clone().into_iter())
            .filter(|(_, sel)| *sel)
            .map(|(r, _)| r)
            .collect();
        saving.set(true);
        save_error.set(None);
        spawn(async move {
            match save_map_transit(id, SaveTransitRequest { routes }).await {
                Ok(_) => {
                    let _ = nav.push(Route::EditMapConfirm { id });
                }
                Err(e) => {
                    saving.set(false);
                    save_error.set(Some(e.to_string()));
                }
            }
        });
    };

    // Derive display state
    let has_routes = !transit_routes.read().is_empty();
    let routes_snap = transit_routes.read().clone();
    let sel_snap = route_selected.read().clone();
    let total = routes_snap.len();
    let n_sel = sel_snap.iter().filter(|&&s| s).count();

    let map_guard = map_res.read();
    let map_name = match &*map_guard {
        Some(Ok(map)) => map.name.clone(),
        _ => String::new(),
    };
    let is_loading = map_guard.is_none() || transit_res.read().is_none();
    let load_error = match (&*map_guard, &*transit_res.read()) {
        (Some(Err(e)), _) | (_, Some(Err(e))) => Some(e.to_string()),
        _ => None,
    };

    if is_loading {
        return rsx! { main { class: "loading", p { "Loading…" } } };
    }
    if let Some(msg) = load_error {
        return rsx! { main { class: "error-page", p { class: "form-error", "{msg}" } } };
    }

    rsx! {
        main { class: "wizard-page",
            WizardSteps { current: 3 }
            div { class: "wizard-body wizard-body--map",

                div { class: "wizard-left",
                    div { class: "wizard-left-scroll",
                        h2 { "{map_name}" }

                        if !has_routes {
                            p { class: "wizard-hint",
                                "Fetch transit routes from OpenStreetMap for this boundary. "
                                "You can then select or deselect individual lines."
                            }
                        }

                        if let Some(msg) = transit_error.read().clone() {
                            p { class: "form-error wizard-error", "{msg}" }
                        }

                        div { class: "transit-fetch-row",
                            button {
                                r#type: "button",
                                class: "btn btn--secondary",
                                disabled: *transit_loading.read(),
                                onclick: fetch_transit_fn,
                                if *transit_loading.read() {
                                    "Fetching…"
                                } else if has_routes {
                                    "Re-fetch from OSM"
                                } else {
                                    "Fetch Transit Routes"
                                }
                            }
                        }

                        if has_routes {
                            p { class: "transit-panel__summary",
                                "{n_sel} of {total} route(s) selected"
                            }

                            div { class: "transit-mode-list",
                                for &mode in MODE_ORDER.iter() {
                                    {
                                        let mode_data: Vec<(usize, String, Option<String>, usize, String)> =
                                            routes_snap.iter().enumerate()
                                                .filter(|(_, r)| r.route_type == mode)
                                                .map(|(i, r)| (
                                                    i,
                                                    r.name.clone(),
                                                    r.long_name.clone(),
                                                    r.stops.len(),
                                                    effective_color(r),
                                                ))
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
                                                        span { class: "mode-swatch", style: "background:{mode_color}" }
                                                        span { class: "transit-mode-header__label", "{mode_label}" }
                                                        span { class: "transit-mode-header__count", "({route_count})" }
                                                    }
                                                    ul { class: "transit-route-list",
                                                        for (global_idx, route_name, long_name, stop_count, color) in mode_data {
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
                                                                            span { class: "route-swatch", style: "background:{color}" }
                                                                            div { class: "transit-route-item__info",
                                                                                strong { "{route_name}" }
                                                                                if let Some(ln) = long_name {
                                                                                    span { class: "transit-route-item__longname", " – {ln}" }
                                                                                }
                                                                                span { class: "transit-route-item__meta", " · {stop_count} stop(s)" }
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
                    }

                    div { class: "wizard-nav",
                        button {
                            r#type: "button",
                            class: "btn btn--ghost",
                            onclick: move |_| { let _ = nav.push(Route::EditMapBoundary { id }); },
                            "← Back"
                        }
                        button {
                            r#type: "button",
                            class: "btn btn--primary",
                            disabled: *saving.read(),
                            onclick: save_and_next,
                            if *saving.read() { "Saving…" } else { "Next: Confirm →" }
                        }
                    }
                }

                // Right: readonly boundary preview + transit overlay
                div { class: "wizard-right",
                    ui::BoundaryMapEditor {
                        boundary,
                        show_waypoints: false,
                        readonly: true,
                        auto_zoom: true,
                    }
                }
            }
        }
    }
}
