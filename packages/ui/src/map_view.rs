use api::{
    endpoints::exclusion_zone::ExclusionZoneResponse,
    types::{
        area::{Area, Polygon},
        transit::TransitData,
    },
};
use dioxus::prelude::*;
use uuid::Uuid;

const LEAFLET_CSS: Asset = asset!("/assets/leaflet.css");
const LEAFLET_JS: Asset = asset!("/assets/leaflet.js");

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
    Object.keys(z).forEach(function(id){
        z[id].setStyle({fillColor:'#1e1e50',fillOpacity:1,stroke:false});
    });
}
"#;

const TRANSIT_DEFAULT_COLORS: &str = r#"
var _transitColors={
    bus:'#1a73e8',tram:'#e8a81a',subway:'#dc3545',train:'#198754',
    ferry:'#0dcaf0',light_rail:'#6f42c1',monorail:'#fd7e14',
    trolleybus:'#20c997',funicular:'#d63384'
};
"#;

#[component]
pub fn MapView(
    boundary: Polygon,
    zones: Signal<Vec<ExclusionZoneResponse>>,
    transit: Signal<Option<TransitData>>,
) -> Element {
    // Initialise Leaflet after first render — async-poll for CDN load
    use_effect(move || {
        if boundary.vertices.is_empty() {
            return;
        }

        let pts_json = serde_json::to_string(&boundary.vertices).unwrap_or_default();
        let init_js = format!(
            r#"
            (async function() {{
                var tries = 0;
                while (typeof L === 'undefined' && tries++ < 100) {{
                    await new Promise(r => setTimeout(r, 100));
                }}
                if (typeof L === 'undefined') return;
                if (window._hideseekMap) return;
                var pts = {pts_json};
                window._hideseekBoundary = pts;
                var lats = pts.map(function(p){{return p.lat;}});
                var lngs = pts.map(function(p){{return p.lng;}});
                var swLat = Math.min.apply(null, lats), swLng = Math.min.apply(null, lngs);
                var neLat = Math.max.apply(null, lats), neLng = Math.max.apply(null, lngs);
                var map = L.map('leaflet-map').fitBounds([[swLat, swLng],[neLat, neLng]]);
                L.tileLayer('https://{{s}}.tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png', {{
                    attribution: '&copy; OpenStreetMap contributors'
                }}).addTo(map);
                // Shade everything outside the play area.
                var world = [[-90,-180],[-90,180],[90,180],[90,-180]];
                L.polygon([world, pts], {{
                    fillColor: '#3a3a3a',
                    fillOpacity: 0.65,
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
                var transitPane = map.createPane('transitPane');
                transitPane.style.zIndex = 390;
                var zonesPane = map.createPane('zonesPane');
                zonesPane.style.isolation = 'isolate';
                zonesPane.style.opacity = '0.55';
            }})();
        "#
        );
        let _ = document::eval(&init_js);
    });

    // Sync zones signal → Leaflet layers.
    use_effect(move || {
        let zones_snap = zones.read().clone();

        let mut js = String::new();
        js.push_str("(function sync(){");
        js.push_str("var m=window._hideseekMap;if(!m){setTimeout(sync,100);return;}");
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

            let Area::Circle(circle) = &zone.area else {
                println!("Found non-circle area in MapView; skipping zone {}", id);
                continue;
            };
            let lat = circle.center.lat;
            let lng = circle.center.lng;
            let r = circle.radius;

            if zone.exclude_outside {
                js.push_str(&format!(
                    "if(!z['{id}']&&boundary&&boundary.length>0){{\
                        var ring=_circleRing({lat},{lng},{r},64);\
                        var c=L.polygon([boundary,ring],{{\
                            fillColor:'#1e1e50',fillOpacity:1,\
                            stroke:false,interactive:false,className:'zone-excluded',\
                            pane:'zonesPane'\
                        }}).addTo(m);"
                ));
            } else {
                js.push_str(&format!(
                    "if(!z['{id}']){{\
                        var c=L.circle([{lat},{lng}],{{\
                            radius:{r},fillColor:'#1e1e50',\
                            fillOpacity:1,stroke:false,className:'zone-excluded',\
                            pane:'zonesPane'\
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

    // Render transit routes and stops when transit data arrives.
    use_effect(move || {
        let transit_snap = transit.read().clone();
        let Some(data) = transit_snap else {
            return;
        };

        let routes_json = serde_json::to_string(&data.routes).unwrap_or_default();

        let js = format!(
            r#"(function syncTransit(){{
                var m=window._hideseekMap;
                if(!m){{setTimeout(syncTransit,200);return;}}
                if(window._hideseekTransitLoaded)return;
                window._hideseekTransitLoaded=true;
                {colors}
                var routes={routes_json};
                routes.forEach(function(route){{
                    var color=route.color||_transitColors[route.route_type]||'#888';
                    (route.waypoints||[]).forEach(function(seg){{
                        if(seg.length>1){{
                            var pts=seg.map(function(p){{return[p.lat,p.lng];}});
                            L.polyline(pts,{{
                                color:color,weight:3,opacity:0.8,
                                interactive:false,pane:'transitPane'
                            }}).bindTooltip(route.name).addTo(m);
                        }}
                    }});
                    (route.stops||[]).forEach(function(stop){{
                        L.circleMarker([stop.lat,stop.lng],{{
                            radius:5,color:'#fff',weight:1.5,
                            fillColor:color,fillOpacity:1,
                            pane:'transitPane'
                        }}).bindTooltip(stop.name).addTo(m);
                    }});
                }});
            }})();"#,
            colors = TRANSIT_DEFAULT_COLORS,
            routes_json = routes_json,
        );
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
                z[tid].setStyle({{color:'#e67e22',fillColor:'#e67e22',fillOpacity:1,weight:3,stroke:true}});
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
