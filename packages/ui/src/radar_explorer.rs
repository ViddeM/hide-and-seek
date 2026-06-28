use api::models::{AddZoneRequest, ExclusionZone};
use crate::map_view::js_highlight_zone;
use dioxus::prelude::*;
use uuid::Uuid;

/// Radar question explorer — shows a live preview circle on `window._hideseekMap`.
/// Seekers see a lock-in button to commit each answer; locked answers accumulate in
/// an "Asked Questions" list from which individual zones can be highlighted.
#[component]
pub fn RadarExplorer(
    game_id: Uuid,
    is_seeker: bool,
    on_zone_added: EventHandler<ExclusionZone>,
) -> Element {
    let mut expanded = use_signal(|| false);
    let mut center: Signal<Option<[f64; 2]>> = use_signal(|| None);
    let mut radius_km = use_signal(|| 5.0f64);
    // true  = Yes: hider IS inside → shade outside the circle
    // false = No:  hider NOT inside → shade the circle itself
    let mut is_yes = use_signal(|| false);
    let mut adding = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut picking_center = use_signal(|| false);
    let mut asked: Signal<Vec<ExclusionZone>> = use_signal(Vec::new);
    let mut highlighted_id: Signal<Option<Uuid>> = use_signal(|| None);

    // Redraw preview whenever center, radius, or Yes/No changes
    use_effect(move || {
        let c = *center.read();
        let r_m = *radius_km.read() * 1000.0;
        let yes = *is_yes.read();

        let js = match c {
            Some([lat, lng]) => {
                if yes {
                    format!(
                        r#"(function(){{
                            var m=window._hideseekMap;if(!m)return;
                            if(window._radarPreview){{m.removeLayer(window._radarPreview);window._radarPreview=null;}}
                            var boundary=window._hideseekBoundary;
                            if(!boundary||boundary.length===0)return;
                            function _cr(lat,lng,r,n){{
                                var R=6371000,pts=[];
                                for(var i=0;i<n;i++){{
                                    var a=2*Math.PI*i/n;
                                    var dlat=r*Math.cos(a)/R*(180/Math.PI);
                                    var dlng=r*Math.sin(a)/(R*Math.cos(lat*Math.PI/180))*(180/Math.PI);
                                    pts.push([lat+dlat,lng+dlng]);
                                }}
                                return pts;
                            }}
                            var ring=_cr({lat},{lng},{r_m},64);
                            window._radarPreview=L.polygon([boundary,ring],{{
                                fillColor:'#1e1e50',fillOpacity:0.35,
                                color:'#1e1e50',dashArray:'8',weight:2,
                                interactive:false,className:'zone-excluded'
                            }}).addTo(m);
                        }})()"#
                    )
                } else {
                    format!(
                        r#"(function(){{
                            var m=window._hideseekMap;if(!m)return;
                            if(window._radarPreview){{m.removeLayer(window._radarPreview);window._radarPreview=null;}}
                            window._radarPreview=L.circle([{lat},{lng}],{{
                                radius:{r_m},color:'#1e1e50',fillColor:'#1e1e50',
                                fillOpacity:0.35,dashArray:'8',weight:2,className:'zone-excluded'
                            }}).addTo(m);
                        }})()"#
                    )
                }
            }
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
                    // ── Question form ──────────────────────────────────────────
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
                            class: if !*is_yes.read() {
                                "btn btn--sm radar-toggle--no radar-toggle--active"
                            } else {
                                "btn btn--sm radar-toggle--no"
                            },
                            r#type: "button",
                            onclick: move |_| is_yes.set(false),
                            "No"
                        }
                        button {
                            class: if *is_yes.read() {
                                "btn btn--sm radar-toggle--yes radar-toggle--active"
                            } else {
                                "btn btn--sm radar-toggle--yes"
                            },
                            r#type: "button",
                            onclick: move |_| is_yes.set(true),
                            "Yes"
                        }
                    }

                    p { class: "radar-explorer__impact",
                        if *is_yes.read() {
                            "Yes → everything outside this circle is excluded."
                        } else {
                            "No → this circle is excluded."
                        }
                    }

                    if is_seeker && center.read().is_some() {
                        div { class: "radar-explorer__apply",
                            button {
                                class: "btn btn--sm btn--primary",
                                r#type: "button",
                                disabled: *adding.read(),
                                onclick: move |_| {
                                    if *adding.read() { return; }
                                    let Some([lat, lng]) = *center.read() else { return; };
                                    let r_m = (*radius_km.read() * 1000.0) as u32;
                                    let yes = *is_yes.read();
                                    adding.set(true);
                                    error.set(None);
                                    let label = if yes { "Radar: Yes" } else { "Radar: No" };
                                    let req = AddZoneRequest {
                                        center_lat: lat,
                                        center_lng: lng,
                                        radius_m: r_m,
                                        exclude_outside: yes,
                                        label: Some(label.to_string()),
                                        question_id: None,
                                    };
                                    spawn(async move {
                                        match api::zones::add_exclusion_zone(game_id, req).await {
                                            Ok(zone) => {
                                                adding.set(false);
                                                // Reset only center so the next question can
                                                // reuse the same radius / Yes-No setting
                                                center.set(None);
                                                asked.write().push(zone.clone());
                                                on_zone_added.call(zone);
                                            }
                                            Err(e) => {
                                                adding.set(false);
                                                error.set(Some(e.to_string()));
                                            }
                                        }
                                    });
                                },
                                if *adding.read() { "Locking in…" } else { "Lock in" }
                            }
                        }
                    }

                    if let Some(msg) = error.read().as_ref() {
                        p { class: "form-error", "{msg}" }
                    }

                    // ── Asked Questions list ───────────────────────────────────
                    if !asked.read().is_empty() {
                        div { class: "asked-questions",
                            h4 { class: "asked-questions__title", "Asked Questions" }

                            for question in asked.read().clone() {
                                {
                                    let qid = question.id;
                                    let is_hl = *highlighted_id.read() == Some(qid);
                                    let radius_km_val = question.radius_m as f64 / 1000.0;
                                    let lat = question.center_lat;
                                    let lng = question.center_lng;
                                    let yes = question.exclude_outside;

                                    rsx! {
                                        div { class: "asked-question",
                                            div { class: "asked-question__info",
                                                span {
                                                    class: if yes {
                                                        "asked-question__answer asked-question__answer--yes"
                                                    } else {
                                                        "asked-question__answer asked-question__answer--no"
                                                    },
                                                    if yes { "Yes" } else { "No" }
                                                }
                                                span { class: "asked-question__detail",
                                                    "{radius_km_val:.1} km · {lat:.3}°, {lng:.3}°"
                                                }
                                            }
                                            button {
                                                class: if is_hl {
                                                    "btn btn--sm btn--primary"
                                                } else {
                                                    "btn btn--sm btn--ghost"
                                                },
                                                r#type: "button",
                                                onclick: move |_| {
                                                    let currently = *highlighted_id.read() == Some(qid);
                                                    let new_hl = if currently { None } else { Some(qid) };
                                                    highlighted_id.set(new_hl);
                                                    let _ = document::eval(&js_highlight_zone(new_hl));
                                                },
                                                if is_hl { "Unhighlight" } else { "Highlight" }
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
