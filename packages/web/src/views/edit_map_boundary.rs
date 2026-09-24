use api::{
    endpoints::maps::{SetBoundaryRequest, get_map, set_map_boundary},
    types::{Point, area::Polygon},
};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::Route;
use super::WizardSteps;

/// Step 2: draw the map boundary.
#[component]
pub fn EditMapBoundary(id: Uuid) -> Element {
    let nav = use_navigator();
    let map_res = use_resource(move || async move { get_map(id).await });

    // Unconditional signals
    let mut boundary: Signal<Vec<Point>> = use_signal(Vec::new);
    let mut error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);
    // Guard so we only overwrite boundary with DB data on first load, not on every re-render
    let mut initialized = use_signal(|| false);

    // Populate boundary from DB once the resource resolves
    use_effect(move || {
        if *initialized.read() {
            return;
        }
        if let Some(Ok(map)) = &*map_res.read() {
            *boundary.write() = map.boundary.vertices.clone();
            initialized.set(true);
        }
    });

    let save_and_next = move |_| {
        let pts = boundary.read().clone();
        if pts.len() < 3 {
            error.set(Some("Draw at least 3 boundary points before continuing.".into()));
            return;
        }
        saving.set(true);
        error.set(None);
        spawn(async move {
            match set_map_boundary(id, SetBoundaryRequest { boundary: Polygon { vertices: pts } }).await {
                Ok(_) => {
                    let _ = nav.push(Route::EditMapTransit { id });
                }
                Err(e) => {
                    saving.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    let guard = map_res.read();
    match &*guard {
        None => rsx! { main { class: "loading", p { "Loading…" } } },
        Some(Err(e)) => {
            let msg = e.to_string();
            rsx! { main { class: "error-page", p { class: "form-error", "{msg}" } } }
        }
        Some(Ok(map)) => {
            let map_name = map.name.clone();
            let pts = boundary.read().clone();

            rsx! {
                main { class: "wizard-page",
                    WizardSteps { current: 2 }
                    div { class: "wizard-body wizard-body--map",
                        div { class: "wizard-left",
                            div { class: "wizard-left-scroll",
                                h2 { "{map_name}" }
                                p { class: "wizard-hint",
                                    "Click on the map to add boundary points. "
                                    "Drag existing points to reposition them. At least 3 required."
                                }

                                if pts.is_empty() {
                                    p { class: "boundary-editor__hint", "No points yet." }
                                } else {
                                    p { class: "boundary-editor__hint",
                                        "{pts.len()} waypoint(s)"
                                        if pts.len() >= 3 { " — polygon ready ✓" }
                                        else { " — need at least 3" }
                                    }
                                    ul { class: "waypoints-list",
                                        for (i, pt) in pts.iter().enumerate() {
                                            {
                                                let lat = pt.lat;
                                                let lng = pt.lng;
                                                rsx! {
                                                    li { class: "waypoint-item", key: "{i}",
                                                        span { class: "waypoint-item__num", "{i+1}" }
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

                                if let Some(msg) = error.read().as_ref() {
                                    p { class: "form-error", "{msg}" }
                                }
                            }

                            div { class: "wizard-nav",
                                button {
                                    r#type: "button",
                                    class: "btn btn--ghost",
                                    onclick: move |_| { let _ = nav.push(Route::EditMapInfo { id }); },
                                    "← Back"
                                }
                                button {
                                    r#type: "button",
                                    class: "btn btn--primary",
                                    disabled: *saving.read() || boundary.read().len() < 3,
                                    onclick: save_and_next,
                                    if *saving.read() { "Saving…" } else { "Next: Transit →" }
                                }
                            }
                        }

                        div { class: "wizard-right",
                            ui::BoundaryMapEditor {
                                boundary,
                                show_waypoints: false,
                                readonly: false,
                                auto_zoom: true,
                            }
                        }
                    }
                }
            }
        }
    }
}
