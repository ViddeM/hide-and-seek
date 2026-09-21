use api::{
    endpoints::{
        game::{CreateGameRequest, CreateGameResponse},
        maps::{CreateMapRequest, MapSummary, PreviewTransitRequest},
    },
    types::{Point, area::Polygon, map_size::MapSize, transit::TransitRoute},
};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::BoundaryMapEditor;

#[derive(Clone, PartialEq)]
enum MapStep {
    Boundary,
    Transit,
}

#[component]
pub fn HostSetupForm(on_created: EventHandler<CreateGameResponse>) -> Element {
    let mut maps = use_resource(api::endpoints::maps::list_maps);

    // Game-creation form state
    let mut host_name = use_signal(String::new);
    let mut selected_map = use_signal(|| None::<Uuid>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    // Map-creation form state
    let mut new_map_name = use_signal(String::new);
    let mut new_map_size = use_signal(|| MapSize::Medium);
    let mut boundary = use_signal(Vec::<Point>::new);
    let mut create_error = use_signal(|| None::<String>);
    let mut create_loading = use_signal(|| false);

    let mut show_create_map = use_signal(|| false);

    // Multi-step wizard state
    let mut map_step = use_signal(|| MapStep::Boundary);
    let mut transit_routes: Signal<Vec<TransitRoute>> = use_signal(Vec::new);
    let mut route_selected: Signal<Vec<bool>> = use_signal(Vec::new);
    let mut transit_loading = use_signal(|| false);
    let mut transit_error = use_signal(|| None::<String>);

    // Draw/clear transit on the boundary editor map whenever step or selection changes
    use_effect(move || {
        let step = map_step.read().clone();
        let routes = transit_routes.read().clone();
        let selected = route_selected.read().clone();

        if step != MapStep::Transit {
            let js = "if(window._bndEditor&&window._bndTransitLayers){window._bndTransitLayers.forEach(function(l){try{window._bndEditor.removeLayer(l);}catch(e){}});window._bndTransitLayers=[];}";
            let _ = document::eval(js);
            return;
        }

        let selected_routes: Vec<&TransitRoute> = routes
            .iter()
            .zip(selected.iter())
            .filter(|(_, sel)| **sel)
            .map(|(r, _)| r)
            .collect();

        let routes_json = serde_json::to_string(&selected_routes).unwrap_or_default();

        let js = format!(
            r#"(function syncTransitEditor(){{
                var m=window._bndEditor;
                if(!m){{setTimeout(syncTransitEditor,200);return;}}
                if(window._bndTransitLayers){{window._bndTransitLayers.forEach(function(l){{try{{m.removeLayer(l);}}catch(e){{}}}});}}
                window._bndTransitLayers=[];
                var colors={{bus:'#1a73e8',tram:'#e8a81a',subway:'#dc3545',train:'#198754',ferry:'#0dcaf0',light_rail:'#6f42c1',monorail:'#fd7e14',trolleybus:'#20c997',funicular:'#d63384'}};
                var routes={routes_json};
                routes.forEach(function(r){{
                    var color=r.color||colors[r.route_type]||'#888';
                    if(r.waypoints&&r.waypoints.length>1){{
                        var pts=r.waypoints.map(function(p){{return[p.lat,p.lng];}});
                        var line=L.polyline(pts,{{color:color,weight:3,opacity:0.85,interactive:false}}).addTo(m);
                        line.bindTooltip(r.name);
                        window._bndTransitLayers.push(line);
                    }}
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
        let size = *new_map_size.read();
        let boundary_pts = boundary.read().clone();

        transit_loading.set(true);
        transit_error.set(None);

        spawn(async move {
            match api::endpoints::maps::preview_transit(PreviewTransitRequest {
                size,
                bounds: Polygon { vertices: boundary_pts },
            })
            .await
            {
                Ok(data) => {
                    let n = data.routes.len();
                    transit_routes.set(data.routes);
                    route_selected.set(vec![true; n]);
                    transit_loading.set(false);
                    map_step.set(MapStep::Transit);
                }
                Err(e) => {
                    transit_loading.set(false);
                    transit_error.set(Some(e.to_string()));
                }
            }
        });
    };

    let mut do_save_map = move |routes: Vec<TransitRoute>| {
        if *create_loading.read() {
            return;
        }
        let name_val = new_map_name.read().trim().to_string();
        if name_val.is_empty() {
            create_error.set(Some("Map name is required".to_string()));
            return;
        }
        let boundary_pts = boundary.read().clone();
        if boundary_pts.len() < 3 {
            create_error.set(Some("Place at least 3 waypoints on the map".to_string()));
            return;
        }

        create_error.set(None);
        create_loading.set(true);
        let size = *new_map_size.read();

        spawn(async move {
            let req = CreateMapRequest {
                name: name_val,
                size,
                bounds: Polygon { vertices: boundary_pts },
                transit_routes: routes,
            };
            match api::endpoints::maps::create_map(req).await {
                Ok(map) => {
                    create_loading.set(false);
                    selected_map.set(Some(map.id));
                    show_create_map.set(false);
                    new_map_name.set(String::new());
                    boundary.write().clear();
                    new_map_size.set(MapSize::Medium);
                    map_step.set(MapStep::Boundary);
                    transit_routes.set(Vec::new());
                    route_selected.set(Vec::new());
                    transit_error.set(None);
                    maps.restart();
                }
                Err(e) => {
                    create_loading.set(false);
                    create_error.set(Some(e.to_string()));
                }
            }
        });
    };

    let submit_with_transit = move |_| {
        let routes_snap = transit_routes.read().clone();
        let sel_snap = route_selected.read().clone();
        let selected: Vec<TransitRoute> = routes_snap
            .into_iter()
            .zip(sel_snap.into_iter())
            .filter(|(_, sel)| *sel)
            .map(|(r, _)| r)
            .collect();
        do_save_map(selected);
    };

    let submit_without_transit = move |_| {
        do_save_map(vec![]);
    };

    let submit_game = move |evt: Event<FormData>| {
        evt.prevent_default();
        if *loading.read() {
            return;
        }
        let name_val = host_name.read().trim().to_string();
        let map_val = *selected_map.read();

        if name_val.is_empty() {
            error.set(Some("Enter your name".to_string()));
            return;
        }
        let Some(map_id) = map_val else {
            error.set(Some("Select a map".to_string()));
            return;
        };
        error.set(None);
        loading.set(true);

        spawn(async move {
            match api::endpoints::game::create_game(CreateGameRequest {
                map_id,
                host_display_name: name_val,
            })
            .await
            {
                Ok(resp) => {
                    loading.set(false);
                    on_created.call(resp);
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    let can_fetch_transit = boundary.read().len() >= 3 && !new_map_name.read().trim().is_empty();
    let is_transit_step = *map_step.read() == MapStep::Transit;

    rsx! {
        main { class: "host-setup",
            h1 { "Host a New Game" }

            form { onsubmit: submit_game,
                label { r#for: "host-name", "Your Name" }
                input {
                    id: "host-name",
                    r#type: "text",
                    placeholder: "Host",
                    oninput: move |e| host_name.set(e.value()),
                    value: host_name.read().clone(),
                }

                label { "Select Map" }

                match &*maps.read() {
                    None => rsx! {
                        p { class: "map-loading", "Loading maps…" }
                    },
                    Some(Err(e)) => {
                        rsx! {
                            p { class: "form-error", "Failed to load maps: {e}" }
                        }
                    }
                    Some(Ok(map_list)) => {
                        let all_empty = map_list.maps.is_empty();
                        rsx! {
                            if all_empty {
                                p { class: "map-empty-hint", "No maps yet — use the form below to create your first one." }
                            }
                            ul { class: "map-list",
                                for map in map_list.maps.iter() {
                                    MapOption {
                                        key: "{map.id}",
                                        map: map.clone(),
                                        selected: *selected_map.read() == Some(map.id),
                                        on_select: move |id| selected_map.set(Some(id)),
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "create-map-toggle-row",
                    button {
                        r#type: "button",
                        class: "btn btn--ghost create-map-toggle",
                        onclick: move |_| {
                            let v = *show_create_map.read();
                            if v {
                                // cancelling — reset wizard state
                                map_step.set(MapStep::Boundary);
                                transit_routes.set(Vec::new());
                                route_selected.set(Vec::new());
                                transit_error.set(None);
                                boundary.write().clear();
                            }
                            show_create_map.set(!v);
                        },
                        if *show_create_map.read() { "✕ Cancel" } else { "+ Create New Map" }
                    }
                }

                if *show_create_map.read() {
                    div { class: "create-map-form",
                        h3 { class: "create-map-form__title",
                            if is_transit_step {
                                "New Map — Step 2: Transit"
                            } else {
                                "New Map — Step 1: Boundary"
                            }
                        }

                        // Step 1 controls (always shown so BoundaryMapEditor stays mounted)
                        div {
                            style: if is_transit_step { "display:none" } else { "" },

                            label { r#for: "map-name", "Map Name" }
                            input {
                                id: "map-name",
                                r#type: "text",
                                placeholder: "e.g. City Centre",
                                oninput: move |e| new_map_name.set(e.value()),
                                value: new_map_name.read().clone(),
                            }

                            label { r#for: "map-size", "Size" }
                            select {
                                id: "map-size",
                                onchange: move |e| {
                                    new_map_size.set(match e.value().as_str() {
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

                        BoundaryMapEditor { boundary }

                        // Step 1 actions
                        if !is_transit_step {
                            if let Some(msg) = transit_error.read().as_ref() {
                                p { class: "form-error", "{msg}" }
                                div { class: "create-map-actions",
                                    button {
                                        r#type: "button",
                                        class: "btn btn--ghost",
                                        disabled: *create_loading.read(),
                                        onclick: submit_without_transit,
                                        if *create_loading.read() { "Saving…" } else { "Save without transit →" }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn btn--secondary",
                                        disabled: !can_fetch_transit || *transit_loading.read(),
                                        onclick: fetch_transit,
                                        if *transit_loading.read() { "Fetching transit…" } else { "Retry transit fetch →" }
                                    }
                                }
                            } else {
                                div { class: "create-map-actions",
                                    button {
                                        r#type: "button",
                                        class: "btn btn--secondary",
                                        disabled: !can_fetch_transit || *transit_loading.read(),
                                        onclick: fetch_transit,
                                        if *transit_loading.read() { "Fetching transit…" } else { "Fetch Transit →" }
                                    }
                                }
                            }
                        }

                        // Step 2: route checklist + actions
                        if is_transit_step {
                            div { class: "transit-selector",
                                {
                                    let route_count = transit_routes.read().len();
                                    rsx! {
                                        p { class: "transit-selector__count",
                                            "{route_count} route(s) found — uncheck any you don't want."
                                        }
                                    }
                                }

                                ul { class: "transit-route-list",
                                    for (i, route) in transit_routes.read().clone().into_iter().enumerate() {
                                        {
                                            let color = route.color.clone().unwrap_or_else(|| {
                                                match route.route_type {
                                                    api::types::transit::TransitRouteType::Bus => "#1a73e8".to_string(),
                                                    api::types::transit::TransitRouteType::Tram => "#e8a81a".to_string(),
                                                    api::types::transit::TransitRouteType::Subway => "#dc3545".to_string(),
                                                    api::types::transit::TransitRouteType::Train => "#198754".to_string(),
                                                    api::types::transit::TransitRouteType::Ferry => "#0dcaf0".to_string(),
                                                    api::types::transit::TransitRouteType::LightRail => "#6f42c1".to_string(),
                                                    api::types::transit::TransitRouteType::Monorail => "#fd7e14".to_string(),
                                                    api::types::transit::TransitRouteType::Trolleybus => "#20c997".to_string(),
                                                    api::types::transit::TransitRouteType::Funicular => "#d63384".to_string(),
                                                }
                                            });
                                            let is_checked = route_selected.read().get(i).copied().unwrap_or(true);
                                            let stop_count = route.stops.len();
                                            let route_name = route.name.clone();
                                            let long_name = route.long_name.clone();
                                            let route_type = format!("{:?}", route.route_type).to_lowercase();
                                            rsx! {
                                                li { class: "transit-route-item", key: "{i}",
                                                    label { class: "transit-route-item__label",
                                                        input {
                                                            r#type: "checkbox",
                                                            checked: is_checked,
                                                            onchange: move |e| {
                                                                let mut sel = route_selected.write();
                                                                if let Some(v) = sel.get_mut(i) {
                                                                    *v = e.checked();
                                                                }
                                                            },
                                                        }
                                                        div {
                                                            class: "transit-route-item__swatch",
                                                            style: "background:{color};width:12px;height:12px;border-radius:50%;display:inline-block;margin:0 6px;flex-shrink:0;",
                                                        }
                                                        div { class: "transit-route-item__info",
                                                            strong { class: "transit-route-item__name", "{route_name}" }
                                                            if let Some(ln) = long_name {
                                                                span { class: "transit-route-item__longname", " – {ln}" }
                                                            }
                                                            span { class: "transit-route-item__meta",
                                                                " · {route_type} · {stop_count} stop(s)"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                if let Some(msg) = create_error.read().as_ref() {
                                    p { class: "form-error", "{msg}" }
                                }

                                div { class: "create-map-actions",
                                    button {
                                        r#type: "button",
                                        class: "btn btn--ghost",
                                        onclick: move |_| {
                                            map_step.set(MapStep::Boundary);
                                            transit_error.set(None);
                                            create_error.set(None);
                                        },
                                        "← Back"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "btn btn--secondary",
                                        disabled: *create_loading.read(),
                                        onclick: submit_with_transit,
                                        if *create_loading.read() { "Saving…" } else { "Save Map" }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(msg) = error.read().as_ref() {
                    p { class: "form-error", "{msg}" }
                }

                button {
                    r#type: "submit",
                    class: "btn btn--primary",
                    disabled: *loading.read()
                        || host_name.read().trim().is_empty()
                        || selected_map.read().is_none(),
                    if *loading.read() { "Creating…" } else { "Create Game" }
                }
            }
        }
    }
}

#[component]
fn MapOption(map: MapSummary, selected: bool, on_select: EventHandler<Uuid>) -> Element {
    let id = map.id;
    let size_str = map.size.to_string();
    rsx! {
        li {
            class: if selected { "map-option map-option--selected" } else { "map-option" },
            onclick: move |_| on_select.call(id),
            strong { "{map.name}" }
            span { class: "map-option__size", " ({size_str})" }
        }
    }
}
