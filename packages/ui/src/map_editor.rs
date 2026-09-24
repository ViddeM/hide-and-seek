use api::types::Point;
use dioxus::prelude::*;

const LEAFLET_CSS: &str = "https://unpkg.com/leaflet@1.9.4/dist/leaflet.css";
const LEAFLET_JS: &str = "https://unpkg.com/leaflet@1.9.4/dist/leaflet.js";

#[component]
pub fn BoundaryMapEditor(boundary: Signal<Vec<Point>>, #[props(default = true)] show_waypoints: bool) -> Element {
    // One-time init: create the Leaflet map and start the click-to-Dioxus bridge
    use_effect(move || {
        let init_js = r#"
            (async function() {
                var tries = 0;
                while (typeof L === 'undefined' && tries++ < 100) {
                    await new Promise(r => setTimeout(r, 100));
                }
                if (typeof L === 'undefined') return;
                try { if (window._bndEditor) { window._bndEditor.remove(); } } catch(e) {}
                window._bndEditor = null;
                window._bndMarkers = [];
                window._bndPoly = null;
                var div = document.getElementById('boundary-editor-map');
                if (!div) return;
                var map = L.map(div).setView([30, 0], 2);
                L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                    attribution: '© OpenStreetMap contributors'
                }).addTo(map);
                window._bndEditor = map;
            })()
        "#;
        let _ = document::eval(init_js);

        // Long-running task: receive click and drag events from JS
        // Messages: {t:"c",lat,lng} for click (add point), {t:"d",i,lat,lng} for drag (move point)
        spawn(async move {
            let mut eval = document::eval(
                r#"
                (async function() {
                    var tries = 0;
                    while (!window._bndEditor && tries++ < 100) {
                        await new Promise(r => setTimeout(r, 100));
                    }
                    if (!window._bndEditor) return;
                    window._bndEditor.on('click', function(e) {
                        dioxus.send({t:'c', lat:e.latlng.lat, lng:e.latlng.lng});
                    });
                    window._bndDragSend = function(i, lat, lng) {
                        dioxus.send({t:'d', i:i, lat:lat, lng:lng});
                    };
                })()
            "#,
            );
            loop {
                match eval.recv::<serde_json::Value>().await {
                    Ok(val) => {
                        match val.get("t").and_then(|t| t.as_str()) {
                            Some("c") => {
                                let lat = val["lat"].as_f64().unwrap_or(0.0);
                                let lng = val["lng"].as_f64().unwrap_or(0.0);
                                boundary.write().push(Point { lat, lng });
                            }
                            Some("d") => {
                                let i = val["i"].as_u64().unwrap_or(0) as usize;
                                let lat = val["lat"].as_f64().unwrap_or(0.0);
                                let lng = val["lng"].as_f64().unwrap_or(0.0);
                                let mut b = boundary.write();
                                if let Some(pt) = b.get_mut(i) {
                                    pt.lat = lat;
                                    pt.lng = lng;
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    });

    // Reactive: redraw polygon whenever boundary changes
    use_effect(move || {
        let pts = boundary.read().clone();
        let pts_json = serde_json::to_string(&pts).unwrap_or_default();
        let redraw_js = format!(
            r#"
            (function() {{
                var m = window._bndEditor;
                if (!m) return;
                var markers = window._bndMarkers || [];
                markers.forEach(function(mk) {{ try {{ m.removeLayer(mk); }} catch(e) {{}} }});
                window._bndMarkers = [];
                if (window._bndPoly) {{ try {{ m.removeLayer(window._bndPoly); }} catch(e) {{}} window._bndPoly = null; }}
                var pts = {pts_json};
                pts.forEach(function(p, i) {{
                    var icon = L.divIcon({{
                        html: '<div class="bnd-wp-icon">' + (i+1) + '</div>',
                        iconSize: [24, 24],
                        iconAnchor: [12, 12],
                        className: ''
                    }});
                    var mk = L.marker([p.lat, p.lng], {{icon: icon, draggable: true}}).addTo(m);
                    mk.on('click', function(e) {{ L.DomEvent.stopPropagation(e); }});
                    mk.on('drag', function() {{
                        if (window._bndPoly) {{
                            var lls = (window._bndMarkers || []).map(function(m) {{ return m.getLatLng(); }});
                            window._bndPoly.setLatLngs(lls);
                        }}
                    }});
                    mk.on('dragend', (function(idx) {{
                        return function(e) {{
                            var ll = e.target.getLatLng();
                            window._bndDragSend && window._bndDragSend(idx, ll.lat, ll.lng);
                        }};
                    }})(i));
                    window._bndMarkers.push(mk);
                }});
                if (pts.length >= 3) {{
                    window._bndPoly = L.polygon(pts, {{color: '#6c63ff', fillOpacity: 0.1, weight: 2}}).addTo(m);
                }} else if (pts.length >= 2) {{
                    window._bndPoly = L.polyline(pts, {{color: '#6c63ff', weight: 2, dashArray: '6'}}).addTo(m);
                }}
            }})()
        "#
        );
        let _ = document::eval(&redraw_js);
    });

    let pts_snapshot = boundary.read().clone();

    rsx! {
        document::Link { rel: "stylesheet", href: LEAFLET_CSS }
        document::Script { src: LEAFLET_JS }

        div { class: "boundary-editor",
            div {
                id: "boundary-editor-map",
                class: "boundary-editor__map",
            }

            if show_waypoints {
                if pts_snapshot.is_empty() {
                    p { class: "boundary-editor__hint", "Click on the map to place boundary waypoints. At least 3 required." }
                } else {
                    p { class: "boundary-editor__hint",
                        "{pts_snapshot.len()} waypoint(s) placed"
                        if pts_snapshot.len() >= 3 { " — polygon ready" }
                        else { " — need at least 3" }
                    }
                    ul { class: "waypoints-list",
                        for (i, pt) in pts_snapshot.iter().enumerate() {
                            {
                                let lat = pt.lat;
                                let lng = pt.lng;
                                rsx! {
                                    li { class: "waypoint-item", key: "{i}",
                                        span { class: "waypoint-item__num", "{i + 1}" }
                                        span { class: "waypoint-item__coords",
                                            "{lat:.4}, {lng:.4}"
                                        }
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
    }
}
