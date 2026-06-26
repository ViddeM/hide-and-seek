use api::models::ExclusionZone;
use dioxus::prelude::*;
use uuid::Uuid;

const LEAFLET_CSS: &str =
    "https://unpkg.com/leaflet@1.9.4/dist/leaflet.css";
const LEAFLET_JS: &str =
    "https://unpkg.com/leaflet@1.9.4/dist/leaflet.js";

#[component]
pub fn MapView(
    boundary: Vec<[f64; 2]>,
    zones: Signal<Vec<ExclusionZone>>,
) -> Element {
    // Initialise Leaflet after first render — async-poll for CDN load
    use_effect(move || {
        if boundary.is_empty() { return; }
        let pts_json = serde_json::to_string(&boundary).unwrap_or_default();
        let init_js = format!(r#"
            (async function() {{
                var tries = 0;
                while (typeof L === 'undefined' && tries++ < 100) {{
                    await new Promise(r => setTimeout(r, 100));
                }}
                if (typeof L === 'undefined') return;
                if (window._hideseekMap) return;
                var pts = {pts_json};
                var lats = pts.map(function(p){{return p[0];}});
                var lngs = pts.map(function(p){{return p[1];}});
                var swLat = Math.min.apply(null, lats), swLng = Math.min.apply(null, lngs);
                var neLat = Math.max.apply(null, lats), neLng = Math.max.apply(null, lngs);
                var map = L.map('leaflet-map').fitBounds([[swLat, swLng],[neLat, neLng]]);
                L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png', {{
                    attribution: '&copy; OpenStreetMap contributors'
                }}).addTo(map);
                L.polygon(pts, {{color: '#6c63ff', fillOpacity: 0.08, weight: 2}}).addTo(map);
                window._hideseekMap = map;
                window._hideseekZones = {{}};
            }})();
        "#);
        let _ = document::eval(&init_js);
    });

    // Sync zones signal → Leaflet circles
    use_effect(move || {
        let zones_snap = zones.read().clone();
        let mut js = String::new();

        // Remove circles no longer in the list
        js.push_str("(function(){var m=window._hideseekMap;if(!m)return;var z=window._hideseekZones||{};");
        js.push_str("var ids={");
        for z in &zones_snap {
            js.push_str(&format!("'{}':1,", z.id));
        }
        js.push_str("};");
        js.push_str("Object.keys(z).forEach(function(id){if(!ids[id]){m.removeLayer(z[id]);delete z[id];}});");

        // Add new circles
        for z in &zones_snap {
            let color = if z.exclude_outside { "#e74c3c" } else { "#3498db" };
            let label = z.label.as_deref().unwrap_or("");
            js.push_str(&format!(
                "if(!z['{id}']){{var c=L.circle([{lat},{lng}],{{radius:{r},color:'{color}',fillOpacity:0.15}}).addTo(m);",
                id = z.id,
                lat = z.center_lat,
                lng = z.center_lng,
                r = z.radius_m,
            ));
            if !label.is_empty() {
                js.push_str(&format!("c.bindTooltip('{label}');"));
            }
            js.push_str(&format!("z['{}'] = c;}}", z.id));
        }
        js.push_str("window._hideseekZones=z;})();");

        let _ = document::eval(&js);
    });

    rsx! {
        document::Link { rel: "stylesheet", href: LEAFLET_CSS }
        document::Script { src: LEAFLET_JS }
        div {
            id: "leaflet-map",
            style: "width:100%;height:100%;min-height:300px;"
        }
    }
}

/// Called from seeker view to add a zone via JS (for optimistic UI).
#[allow(dead_code)]
pub fn js_add_zone(zone: &ExclusionZone) -> String {
    let color = if zone.exclude_outside { "#e74c3c" } else { "#3498db" };
    let label = zone.label.as_deref().unwrap_or("");
    let mut js = format!(
        "(function(){{var m=window._hideseekMap;if(!m)return;var z=window._hideseekZones||{{}};",
    );
    js.push_str(&format!(
        "var c=L.circle([{lat},{lng}],{{radius:{r},color:'{color}',fillOpacity:0.15}}).addTo(m);",
        lat = zone.center_lat,
        lng = zone.center_lng,
        r = zone.radius_m,
    ));
    if !label.is_empty() {
        js.push_str(&format!("c.bindTooltip('{label}');"));
    }
    js.push_str(&format!("z['{}'] = c;window._hideseekZones=z;}})();", zone.id));
    js
}

/// Remove a zone circle from Leaflet.
#[allow(dead_code)]
pub fn js_remove_zone(zone_id: Uuid) -> String {
    format!(
        "(function(){{var z=window._hideseekZones;if(z&&z['{id}']){{window._hideseekMap.removeLayer(z['{id}']);delete z['{id}'];}}}})();",
        id = zone_id,
    )
}
