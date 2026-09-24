use api::{
    endpoints::maps::{CreateDraftMapRequest, create_draft_map},
    types::map_size::MapSize,
};
use dioxus::prelude::*;

use crate::Route;
use super::WizardSteps;

/// Step 1 of map creation: choose a name and size, then create the draft in the DB.
#[component]
pub fn NewMap() -> Element {
    let nav = use_navigator();
    let mut name = use_signal(String::new);
    let mut size = use_signal(|| MapSize::Medium);
    let mut error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);

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
            match create_draft_map(CreateDraftMapRequest { name: name_val, size: size_val }).await {
                Ok(map) => {
                    let _ = nav.push(Route::EditMapBoundary { id: map.id });
                }
                Err(e) => {
                    saving.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
        main { class: "wizard-page",
            WizardSteps { current: 1 }
            div { class: "wizard-body wizard-body--single",
                h2 { "Name your map" }
                div { class: "wizard-left-scroll",
                    label { r#for: "map-name", "Map Name" }
                    input {
                        id: "map-name",
                        r#type: "text",
                        placeholder: "e.g. Gothenburg Central",
                        autofocus: true,
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
                        if *saving.read() { "Creating…" } else { "Next: Draw Boundary →" }
                    }
                }
            }
        }
    }
}
