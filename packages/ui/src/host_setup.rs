use api::{
    endpoints::{
        game::{CreateGameRequest, CreateGameResponse},
        maps::MapSummary,
    },
    types::{map_size::MapSize, map_status::MapStatus},
};
use dioxus::prelude::*;
use uuid::Uuid;

#[component]
pub fn HostSetupForm(
    on_created: EventHandler<CreateGameResponse>,
    on_create_map: EventHandler<()>,
    on_continue_draft: EventHandler<Uuid>,
) -> Element {
    let mut maps = use_resource(api::endpoints::maps::list_maps);

    let mut host_name = use_signal(String::new);
    let mut selected_map = use_signal(|| None::<Uuid>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

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
                        let complete: Vec<&MapSummary> = map_list.maps.iter()
                            .filter(|m| m.status == MapStatus::Complete)
                            .collect();
                        let drafts: Vec<&MapSummary> = map_list.maps.iter()
                            .filter(|m| m.status == MapStatus::Draft)
                            .collect();

                        rsx! {
                            if complete.is_empty() {
                                p { class: "map-empty-hint",
                                    "No published maps yet — create and publish one below."
                                }
                            } else {
                                ul { class: "map-list",
                                    for map in complete.iter() {
                                        MapOption {
                                            key: "{map.id}",
                                            map: (*map).clone(),
                                            selected: *selected_map.read() == Some(map.id),
                                            on_select: move |id| selected_map.set(Some(id)),
                                        }
                                    }
                                }
                            }

                            if !drafts.is_empty() {
                                div { class: "draft-maps-section",
                                    p { class: "draft-maps-label", "Drafts (unfinished)" }
                                    ul { class: "map-list map-list--drafts",
                                        for draft in drafts.iter() {
                                            {
                                                let draft_id = draft.id;
                                                let size_str = draft.size.to_string();
                                                rsx! {
                                                    li { class: "map-option map-option--draft",
                                                        strong { "{draft.name}" }
                                                        span { class: "map-option__size", " ({size_str})" }
                                                        span { class: "map-option__badge", "Draft" }
                                                        button {
                                                            r#type: "button",
                                                            class: "btn btn--ghost btn--sm",
                                                            onclick: move |_| on_continue_draft.call(draft_id),
                                                            "Continue editing →"
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

                div { class: "create-map-toggle-row",
                    button {
                        r#type: "button",
                        class: "btn btn--ghost create-map-toggle",
                        onclick: move |_| on_create_map.call(()),
                        "+ Create New Map"
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
