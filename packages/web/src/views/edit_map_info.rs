use api::{
    endpoints::maps::{UpdateMapInfoRequest, get_map, update_map_info},
    types::map_size::MapSize,
};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::Route;
use super::WizardSteps;

/// Step 1 (edit mode): update the name/size of an existing map.
#[component]
pub fn EditMapInfo(id: Uuid) -> Element {
    let nav = use_navigator();
    let map_res = use_resource(move || async move { get_map(id).await });

    // Unconditional signals — populated from DB once loaded
    let mut name = use_signal(String::new);
    let mut size = use_signal(|| MapSize::Medium);
    let mut error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);
    let mut initialized = use_signal(|| false);

    use_effect(move || {
        if *initialized.read() {
            return;
        }
        if let Some(Ok(map)) = &*map_res.read() {
            name.set(map.name.clone());
            size.set(map.size);
            initialized.set(true);
        }
    });

    let submit = move |_: MouseEvent| {
        let name_val = name.read().trim().to_string();
        if name_val.is_empty() {
            error.set(Some("Map name is required.".into()));
            return;
        }
        let size_val = *size.read();
        saving.set(true);
        error.set(None);
        spawn(async move {
            match update_map_info(id, UpdateMapInfoRequest { name: name_val, size: size_val }).await {
                Ok(_) => {
                    let _ = nav.push(Route::EditMapBoundary { id });
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
        Some(Ok(_)) => {
            rsx! {
                main { class: "wizard-page",
                    WizardSteps { current: 1 }
                    div { class: "wizard-body wizard-body--single",
                        h2 { "Edit map info" }
                        div { class: "wizard-left-scroll",
                            label { r#for: "map-name", "Map Name" }
                            input {
                                id: "map-name",
                                r#type: "text",
                                value: name.read().clone(),
                                oninput: move |e| name.set(e.value()),
                            }

                            label { "Map Size" }
                            div { class: "size-options",
                                for (val, lbl, desc) in [
                                    (MapSize::Small, "Small", "Neighbourhood / city district"),
                                    (MapSize::Medium, "Medium", "City / metro region"),
                                    (MapSize::Large, "Large", "Region / country"),
                                ] {
                                    {
                                        let selected = *size.read() == val;
                                        rsx! {
                                            label {
                                                class: if selected { "size-option size-option--selected" } else { "size-option" },
                                                input {
                                                    r#type: "radio",
                                                    name: "map-size",
                                                    checked: selected,
                                                    onchange: move |_| size.set(val),
                                                }
                                                strong { "{lbl}" }
                                                span { "{desc}" }
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
                            a { href: "/host", class: "btn btn--ghost", "← Cancel" }
                            button {
                                r#type: "button",
                                class: "btn btn--primary",
                                disabled: *saving.read() || name.read().trim().is_empty(),
                                onclick: submit,
                                if *saving.read() { "Saving…" } else { "Next: Boundary →" }
                            }
                        }
                    }
                }
            }
        }
    }
}
