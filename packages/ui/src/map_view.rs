use api::models::ExclusionZone;
use dioxus::prelude::*;
use uuid::Uuid;

const LEAFLET_CSS: &str =
    "https://unpkg.com/leaflet@1.9.4/dist/leaflet.css";
const LEAFLET_JS: &str =
    "https://unpkg.com/leaflet@1.9.4/dist/leaflet.js";

/// Converts a circle (lat/lng centre, radius in metres) to an N-point lat/lng ring.
const CIRCLE_RING_FN: &str = r#"
function _circleRing(lat,lng,r,n){
    var R=6371000,pts=[];
    for(var i=0;i<n;i++){
        var a=2*Math.PI*i/n;
        var dlat=r*Math.cos(a)/R*(180/Math.PI);
        var dlng=r*Math.sin(a)/(R*Math.cos(lat*Math.PI/180))*(180/Math.PI);
        pts.push([lat+dlat,lng+dlng]);
    }
    return pts;
}
"#;

/// Restore all zones to their base style (called before applying a highlight).
pub const RESTORE_ZONES_FN: &str = r#"
function _restoreZones(){
    var z=window._hideseekZones||{};
    var meta=window._hideseekZoneMeta||{};
    Object.keys(z).forEach(function(id){
        var m=meta[id];if(!m)return;
        if(m.eo){
            z[id].setStyle({fillColor:'#1e1e50',fillOpacity:0.45,stroke:false,weight:0});
        }else{
            z[id].setStyle({color:'#1e1e50',fillColor:'#1e1e50',fillOpacity:0.55,weight:2,stroke:true});
        }
    });
}
"#;

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
                window._hideseekBoundary = pts;
                var lats = pts.map(function(p){{return p[0];}});
                var lngs = pts.map(function(p){{return p[1];}});
                var swLat = Math.min.apply(null, lats), swLng = Math.min.apply(null, lngs);
                var neLat = Math.max.apply(null, lats), neLng = Math.max.apply(null, lngs);
                var map = L.map('leaflet-map').fitBounds([[swLat, swLng],[neLat, neLng]]);
                L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png', {{
                    attribution: '&copy; OpenStreetMap contributors'
                }}).addTo(map);
                // Shade everything outside the play area
                var world = [[-90,-180],[-90,180],[90,180],[90,-180]];
                L.polygon([world, pts], {{
                    fillColor: '#1e1e50',
                    fillOpacity: 0.45,
                    stroke: false,
                    interactive: false,
                    className: 'outside-boundary'
                }}).addTo(map);
                // Crisp play-area border
                L.polygon(pts, {{
                    color: '#6c63ff',
                    weight: 2,
                    fillOpacity: 0,
                    interactive: false
                }}).addTo(map);
                window._hideseekMap = map;
                window._hideseekZones = {{}};
                window._hideseekZoneMeta = {{}};
            }})();
        "#);
        let _ = document::eval(&init_js);
    });

    // Sync zones signal → Leaflet layers.
    // Every zone — whether Yes (exclude outside) or No (exclude circle) — gets its own
    // named layer stored in _hideseekZones[id] so it can be highlighted individually.
    use_effect(move || {
        let zones_snap = zones.read().clone();

        let mut js = String::new();
        js.push_str("(function(){");
        js.push_str("var m=window._hideseekMap;if(!m)return;");
        js.push_str("var boundary=window._hideseekBoundary;");
        js.push_str(CIRCLE_RING_FN);
        js.push_str("var z=window._hideseekZones||{};");
        js.push_str("var meta=window._hideseekZoneMeta||{};");

        // Build set of current zone IDs and remove any stale layers
        js.push_str("var ids={");
        for zone in &zones_snap {
            js.push_str(&format!("'{}':1,", zone.id));
        }
        js.push_str("};");
        js.push_str("Object.keys(z).forEach(function(id){if(!ids[id]){m.removeLayer(z[id]);delete z[id];delete meta[id];}});");

        // Add any zones not yet on the map
        for zone in &zones_snap {
            let label = zone.label.as_deref().unwrap_or("").replace('\'', "\\'");
            let id = zone.id;
            let lat = zone.center_lat;
            let lng = zone.center_lng;
            let r = zone.radius_m;

            if zone.exclude_outside {
                // Yes zone: boundary polygon with this circle cut out as a hole.
                // mix-blend-mode: darken (via CSS class) means multiple such polygons
                // don't compound — the union looks like one flat shaded area.
                js.push_str(&format!(
                    "if(!z['{id}']&&boundary&&boundary.length>0){{\
                        var ring=_circleRing({lat},{lng},{r},64);\
                        var c=L.polygon([boundary,ring],{{\
                            fillColor:'#1e1e50',fillOpacity:0.45,\
                            stroke:false,interactive:false,className:'zone-excluded'\
                        }}).addTo(m);"
                ));
            } else {
                // No zone: simple shaded circle.
                js.push_str(&format!(
                    "if(!z['{id}']){{\
                        var c=L.circle([{lat},{lng}],{{\
                            radius:{r},color:'#1e1e50',fillColor:'#1e1e50',\
                            fillOpacity:0.55,weight:2,className:'zone-excluded'\
                        }}).addTo(m);"
                ));
            }
            if !label.is_empty() {
                js.push_str(&format!("c.bindTooltip('{label}');"));
            }
            let eo = zone.exclude_outside;
            js.push_str(&format!(
                "z['{id}']=c; meta['{id}']={{eo:{eo}}};\
                }}"
            ));
        }

        js.push_str("window._hideseekZones=z; window._hideseekZoneMeta=meta;");
        js.push_str("})();");
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

/// Highlight one zone by id (pass empty string to clear all highlights).
pub fn js_highlight_zone(zone_id: Option<Uuid>) -> String {
    let id_str = zone_id.map(|id| id.to_string()).unwrap_or_default();
    format!(
        r#"(function(){{
            {restore}
            var tid='{id}';
            var z=window._hideseekZones||{{}};
            if(tid&&z[tid]){{
                z[tid].setStyle({{color:'#e67e22',fillColor:'#e67e22',fillOpacity:0.70,weight:3,stroke:true}});
                z[tid].bringToFront();
            }}
        }})()"#,
        restore = RESTORE_ZONES_FN,
        id = id_str,
    )
}

/// Remove a zone from the Leaflet map.
#[allow(dead_code)]
pub fn js_remove_zone(zone_id: Uuid) -> String {
    format!(
        "(function(){{\
            var z=window._hideseekZones;\
            if(z&&z['{id}']){{window._hideseekMap.removeLayer(z['{id}']);delete z['{id}'];}}\
            var meta=window._hideseekZoneMeta;\
            if(meta){{delete meta['{id}'];}}\
        }})();",
        id = zone_id,
    )
}
