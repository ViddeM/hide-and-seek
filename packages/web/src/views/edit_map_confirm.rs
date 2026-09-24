use api::endpoints::maps::{finalize_map, get_map, get_transit};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::Route;
use super::WizardSteps;

/// Step 4: review the map and publish it.
#[component]
pub fn EditMapConfirm(id: Uuid) -> Element {
    let nav = use_navigator();

    let map_res = use_resource(move || async move { get_map(id).await });
    let transit_res = use_resource(move || async move { get_transit(id).await });

    // Unconditional signals
    let mut publishing = use_signal(|| false);
    let mut publish_error = use_signal(|| None::<String>);

    let publish = move |_| {
        if *publishing.read() {
            return;
        }
        publishing.set(true);
        publish_error.set(None);
        spawn(async move {
            match finalize_map(id).await {
                Ok(_) => {
                    let _ = nav.push(Route::HostSetup {});
                }
                Err(e) => {
                    publishing.set(false);
                    publish_error.set(Some(e.to_string()));
                }
            }
        });
    };

    let map_guard = map_res.read();
    let transit_guard = transit_res.read();

    match (&*map_guard, &*transit_guard) {
        (None, _) | (_, None) => {
            rsx! { main { class: "loading", p { "Loading…" } } }
        }
        (Some(Err(e)), _) | (_, Some(Err(e))) => {
            let msg = e.to_string();
            rsx! { main { class: "error-page", p { class: "form-error", "{msg}" } } }
        }
        (Some(Ok(map)), Some(Ok(transit))) => {
            let map_name = map.name.clone();
            let map_size = map.size.to_string();
            let boundary_count = map.boundary.vertices.len();
            let route_count = transit.routes.len();
            let stop_count: usize = transit.routes.iter().map(|r| r.stops.len()).sum();

            rsx! {
                main { class: "wizard-page",
                    WizardSteps { current: 4 }
                    div { class: "wizard-body wizard-body--single",
                        h2 { "Review & Publish" }
                        div { class: "wizard-left-scroll",
                            div { class: "confirm-summary",
                                div { class: "confirm-row",
                                    span { class: "confirm-label", "Name" }
                                    span { class: "confirm-value", "{map_name}" }
                                }
                                div { class: "confirm-row",
                                    span { class: "confirm-label", "Size" }
                                    span { class: "confirm-value", "{map_size}" }
                                }
                                div { class: "confirm-row",
                                    span { class: "confirm-label", "Boundary" }
                                    span { class: "confirm-value", "{boundary_count} waypoints" }
                                }
                                div { class: "confirm-row",
                                    span { class: "confirm-label", "Transit" }
                                    span { class: "confirm-value",
                                        if route_count == 0 {
                                            "No transit routes"
                                        } else {
                                            "{route_count} route(s) · {stop_count} stop(s)"
                                        }
                                    }
                                }
                            }

                            p { class: "wizard-hint",
                                "Publishing makes this map available for games. "
                                "You can still edit it afterwards from the host setup."
                            }

                            if let Some(msg) = publish_error.read().clone() {
                                p { class: "form-error", "{msg}" }
                            }
                        }

                        div { class: "wizard-nav",
                            button {
                                r#type: "button",
                                class: "btn btn--ghost",
                                onclick: move |_| { let _ = nav.push(Route::EditMapTransit { id }); },
                                "← Back"
                            }
                            button {
                                r#type: "button",
                                class: "btn btn--primary",
                                disabled: *publishing.read(),
                                onclick: publish,
                                if *publishing.read() { "Publishing…" } else { "Publish Map" }
                            }
                        }
                    }
                }
            }
        }
    }
}
