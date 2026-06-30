use api::{
    endpoints::{
        game::{CreateGameRequest, CreateGameResponse},
        maps::{CreateMapRequest, MapSummary},
    },
    types::{Point, area::Polygon, map_size::MapSize},
};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::BoundaryMapEditor;

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

    let submit_create_map = move |_| {
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
                bounds: Polygon {
                    vertices: boundary_pts,
                },
            };
            match api::endpoints::maps::create_map(req).await {
                Ok(map) => {
                    create_loading.set(false);
                    selected_map.set(Some(map.id));
                    show_create_map.set(false);
                    new_map_name.set(String::new());
                    boundary.write().clear();
                    new_map_size.set(MapSize::Medium);
                    maps.restart();
                }
                Err(e) => {
                    create_loading.set(false);
                    create_error.set(Some(e.to_string()));
                }
            }
        });
    };

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
                        // let extra = new_maps.read().clone();
                        let extra = vec![];
                        let all_empty = map_list.maps.is_empty() && extra.is_empty();
                        rsx! {
                            if all_empty {
                                p { class: "map-empty-hint", "No maps yet — use the form below to create your first one." }
                            }
                            ul { class: "map-list",
                                for map in map_list.maps.iter().chain(extra.iter()) {
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
            }

            // Toggle for the inline map-creation form
            div { class: "create-map-toggle-row",
                button {
                    r#type: "button",
                    class: "btn btn--ghost create-map-toggle",
                    onclick: move |_| {
                        let v = *show_create_map.read();
                        show_create_map.set(!v);
                    },
                    if *show_create_map.read() {
                        "✕ Cancel"
                    } else {
                        "+ Create New Map"
                    }
                }
            }

            // Inline map-creation form (div, not a nested <form>)
            if *show_create_map.read() {
                div { class: "create-map-form",
                    h3 { class: "create-map-form__title", "New Map" }

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
                            new_map_size
                                .set(
                                    match e.value().as_str() {
                                        "small" => MapSize::Small,
                                        "large" => MapSize::Large,
                                        _ => MapSize::Medium,
                                    },
                                );
                        },
                        option { value: "small", "Small" }
                        option { value: "medium", selected: true, "Medium" }
                        option { value: "large", "Large" }
                    }

                    BoundaryMapEditor { boundary }

                    if let Some(msg) = create_error.read().as_ref() {
                        p { class: "form-error", "{msg}" }
                    }

                    button {
                        r#type: "button",
                        class: "btn btn--secondary",
                        disabled: *create_loading.read(),
                        onclick: submit_create_map,
                        if *create_loading.read() {
                            "Saving…"
                        } else {
                            "Save Map"
                        }
                    }
                }


                if let Some(msg) = error.read().as_ref() {
                    p { class: "form-error", "{msg}" }
                }

                button {
                    r#type: "submit",
                    class: "btn btn--primary",
                    disabled: *loading.read(),
                    if *loading.read() {
                        "Creating…"
                    } else {
                        "Create Game"
                    }
                }
            }
        }
    }

    // let mut host_name = use_signal(String::new);
    // let mut host_role = use_signal(|| TeamRole::Hider);
    // let mut selected_map = use_signal(|| None::<Uuid>);
    // let mut error = use_signal(|| None::<String>);
    // let mut loading = use_signal(|| false);

    // // Maps created this session — appended locally so we don't need to restart the resource
    // let mut new_maps = use_signal(Vec::<MapSummary>::new);
    // let mut show_create_map = use_signal(|| false);

    // // Map-creation form state
    // let mut new_map_name = use_signal(String::new);
    // let mut new_map_size = use_signal(|| MapSize::Medium);
    // let mut boundary = use_signal(Vec::<[f64; 2]>::new);
    // let mut create_error = use_signal(|| None::<String>);
    // let mut create_loading = use_signal(|| false);

    // // Auto-open the create-map form when the list comes back empty
    // use_effect(move || {
    //     let loaded_empty = match &*maps.read() {
    //         Some(Ok(list)) => list.is_empty(),
    //         _ => false,
    //     };
    //     if loaded_empty && new_maps.read().is_empty() {
    //         show_create_map.set(true);
    //     }
    // });

    // let submit_game = move |evt: Event<FormData>| {
    //     evt.prevent_default();
    //     if *loading.read() {
    //         return;
    //     }
    //     let name_val = host_name.read().trim().to_string();
    //     let map_val = *selected_map.read();

    //     if name_val.is_empty() {
    //         error.set(Some("Enter your name".to_string()));
    //         return;
    //     }
    //     let Some(map_id) = map_val else {
    //         error.set(Some("Select a map".to_string()));
    //         return;
    //     };
    //     error.set(None);
    //     loading.set(true);
    //     let role = *host_role.read();

    //     spawn(async move {
    //         match api::auth::create_game(map_id, name_val, role).await {
    //             Ok(resp) => {
    //                 loading.set(false);
    //                 on_created.call(resp);
    //             }
    //             Err(e) => {
    //                 loading.set(false);
    //                 error.set(Some(e.to_string()));
    //             }
    //         }
    //     });
    // };

    // let submit_create_map = move |_| {
    //     if *create_loading.read() {
    //         return;
    //     }
    //     let name_val = new_map_name.read().trim().to_string();
    //     if name_val.is_empty() {
    //         create_error.set(Some("Map name is required".to_string()));
    //         return;
    //     }
    //     let boundary_pts = boundary.read().clone();
    //     if boundary_pts.len() < 3 {
    //         create_error.set(Some("Place at least 3 waypoints on the map".to_string()));
    //         return;
    //     }

    //     create_error.set(None);
    //     create_loading.set(true);
    //     let size = *new_map_size.read();

    //     spawn(async move {
    //         let req = CreateMapRequest {
    //             name: name_val,
    //             size,
    //             boundary: boundary_pts,
    //             stops: vec![],
    //             questions: vec![],
    //         };
    //         match api::maps::create_map(req).await {
    //             Ok(map) => {
    //                 create_loading.set(false);
    //                 selected_map.set(Some(map.id));
    //                 new_maps.write().push(map);
    //                 show_create_map.set(false);
    //                 new_map_name.set(String::new());
    //                 boundary.write().clear();
    //                 new_map_size.set(MapSize::Medium);
    //             }
    //             Err(e) => {
    //                 create_loading.set(false);
    //                 create_error.set(Some(e.to_string()));
    //             }
    //         }
    //     });
    // };

    // rsx! {
    //     main { class: "host-setup",
    //         h1 { "Host a New Game" }

    //         form { onsubmit: submit_game,
    //             label { r#for: "host-name", "Your Name" }
    //             input {
    //                 id: "host-name",
    //                 r#type: "text",
    //                 placeholder: "Host",
    //                 oninput: move |e| host_name.set(e.value()),
    //                 value: host_name.read().clone(),
    //             }

    //             fieldset { class: "role-toggle",
    //                 legend { "Your Role" }
    //                 label {
    //                     input {
    //                         r#type: "radio",
    //                         name: "host-role",
    //                         checked: *host_role.read() == TeamRole::Hider,
    //                         onchange: move |_| host_role.set(TeamRole::Hider),
    //                     }
    //                     "Hider"
    //                 }
    //                 label {
    //                     input {
    //                         r#type: "radio",
    //                         name: "host-role",
    //                         checked: *host_role.read() == TeamRole::Seeker,
    //                         onchange: move |_| host_role.set(TeamRole::Seeker),
    //                     }
    //                     "Seeker"
    //                 }
    //             }

    //             label { "Select Map" }

    //             match &*maps.read() {
    //                 None => rsx! {
    //                     p { class: "map-loading", "Loading maps…" }
    //                 },
    //                 Some(Err(e)) => {
    //                     rsx! {
    //                         p { class: "form-error", "Failed to load maps: {e}" }
    //                     }
    //                 }
    //                 Some(Ok(map_list)) => {
    //                     let extra = new_maps.read().clone();
    //                     let all_empty = map_list.is_empty() && extra.is_empty();
    //                     rsx! {
    //                         if all_empty {
    //                             p { class: "map-empty-hint", "No maps yet — use the form below to create your first one." }
    //                         }
    //                         ul { class: "map-list",
    //                             for map in map_list.iter().chain(extra.iter()) {
    //                                 MapOption {
    //                                     key: "{map.id}",
    //                                     map: map.clone(),
    //                                     selected: *selected_map.read() == Some(map.id),
    //                                     on_select: move |id| selected_map.set(Some(id)),
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }

    //             // Toggle for the inline map-creation form
    //             div { class: "create-map-toggle-row",
    //                 button {
    //                     r#type: "button",
    //                     class: "btn btn--ghost create-map-toggle",
    //                     onclick: move |_| {
    //                         let v = *show_create_map.read();
    //                         show_create_map.set(!v);
    //                     },
    //                     if *show_create_map.read() {
    //                         "✕ Cancel"
    //                     } else {
    //                         "+ Create New Map"
    //                     }
    //                 }
    //             }

    //             // Inline map-creation form (div, not a nested <form>)
    //             if *show_create_map.read() {
    //                 div { class: "create-map-form",
    //                     h3 { class: "create-map-form__title", "New Map" }

    //                     label { r#for: "map-name", "Map Name" }
    //                     input {
    //                         id: "map-name",
    //                         r#type: "text",
    //                         placeholder: "e.g. City Centre",
    //                         oninput: move |e| new_map_name.set(e.value()),
    //                         value: new_map_name.read().clone(),
    //                     }

    //                     label { r#for: "map-size", "Size" }
    //                     select {
    //                         id: "map-size",
    //                         onchange: move |e| {
    //                             new_map_size
    //                                 .set(
    //                                     match e.value().as_str() {
    //                                         "small" => MapSize::Small,
    //                                         "large" => MapSize::Large,
    //                                         _ => MapSize::Medium,
    //                                     },
    //                                 );
    //                         },
    //                         option { value: "small", "Small" }
    //                         option { value: "medium", selected: true, "Medium" }
    //                         option { value: "large", "Large" }
    //                     }

    //                     crate::BoundaryMapEditor { boundary }

    //                     if let Some(msg) = create_error.read().as_ref() {
    //                         p { class: "form-error", "{msg}" }
    //                     }

    //                     button {
    //                         r#type: "button",
    //                         class: "btn btn--secondary",
    //                         disabled: *create_loading.read(),
    //                         onclick: submit_create_map,
    //                         if *create_loading.read() {
    //                             "Saving…"
    //                         } else {
    //                             "Save Map"
    //                         }
    //                     }
    //                 }
    //             }

    //             if let Some(msg) = error.read().as_ref() {
    //                 p { class: "form-error", "{msg}" }
    //             }

    //             button {
    //                 r#type: "submit",
    //                 class: "btn btn--primary",
    //                 disabled: *loading.read(),
    //                 if *loading.read() {
    //                     "Creating…"
    //                 } else {
    //                     "Create Game"
    //                 }
    //             }
    //         }
    //     }
    // }
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
