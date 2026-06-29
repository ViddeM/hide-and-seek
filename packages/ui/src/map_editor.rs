use api::types::Point;
use dioxus::prelude::*;

const LEAFLET_CSS: &str = "https://unpkg.com/leaflet@1.9.4/dist/leaflet.css";
const LEAFLET_JS: &str = "https://unpkg.com/leaflet@1.9.4/dist/leaflet.js";

#[component]
pub fn BoundaryMapEditor(boundary: Signal<Vec<Point>>) -> Element {
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

        // Long-running task: receive click events from JS
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
                        dioxus.send([e.latlng.lat, e.latlng.lng]);
                    });
                })()
            "#,
            );
            loop {
                match eval.recv::<[f64; 2]>().await {
                    Ok(pt) => {
                        boundary.write().push(Point {
                            lat: pt[0],
                            lng: pt[1],
                        });
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
                    var mk = L.circleMarker(p, {{
                        radius: 8, fillColor: '#6c63ff', color: '#fff', weight: 2, fillOpacity: 0.9
                    }}).addTo(m);
                    mk.bindTooltip(String(i + 1), {{permanent: true, direction: 'center', className: 'bnd-label'}});
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
