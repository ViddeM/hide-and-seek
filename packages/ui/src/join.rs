use api::models::{JoinGameResponse, TeamRole};
use dioxus::prelude::*;

#[component]
pub fn JoinForm(
    initial_code: Option<String>,
    on_joined: EventHandler<JoinGameResponse>,
) -> Element {
    let mut code = use_signal(|| initial_code.clone().unwrap_or_default().to_uppercase());
    let mut display_name = use_signal(String::new);
    let mut team_name = use_signal(String::new);
    let mut role = use_signal(|| TeamRole::Hider);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        if *loading.read() {
            return;
        }
        let code_val = code.read().trim().to_uppercase().to_string();
        let name_val = display_name.read().trim().to_string();
        let team_val = team_name.read().trim().to_string();
        let role_val = *role.read();

        if code_val.len() != 6 {
            error.set(Some("Enter the 6-character game code".to_string()));
            return;
        }
        if name_val.is_empty() {
            error.set(Some("Enter your display name".to_string()));
            return;
        }
        if team_val.is_empty() {
            error.set(Some("Enter your team name".to_string()));
            return;
        }
        error.set(None);
        loading.set(true);

        spawn(async move {
            match api::auth::join_game(code_val, name_val, team_val, role_val).await {
                Ok(resp) => {
                    loading.set(false);
                    on_joined.call(resp);
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
        main { class: "join-form",
            h1 { "Join Game" }

            form { onsubmit: submit,
                label { r#for: "join-code", "Game Code" }
                input {
                    id: "join-code",
                    r#type: "text",
                    placeholder: "A3X7K2",
                    maxlength: 6,
                    oninput: move |e| code.set(e.value().to_uppercase()),
                    value: code.read().clone(),
                    readonly: initial_code.is_some(),
                }

                label { r#for: "join-name", "Your Name" }
                input {
                    id: "join-name",
                    r#type: "text",
                    placeholder: "Alice",
                    oninput: move |e| display_name.set(e.value()),
                    value: display_name.read().clone(),
                }

                label { r#for: "join-team", "Team Name" }
                input {
                    id: "join-team",
                    r#type: "text",
                    placeholder: "Team Red",
                    oninput: move |e| team_name.set(e.value()),
                    value: team_name.read().clone(),
                }

                fieldset { class: "role-toggle",
                    legend { "Role" }
                    label {
                        input {
                            r#type: "radio",
                            name: "role",
                            value: "hider",
                            checked: *role.read() == TeamRole::Hider,
                            onchange: move |_| role.set(TeamRole::Hider),
                        }
                        "Hider"
                    }
                    label {
                        input {
                            r#type: "radio",
                            name: "role",
                            value: "seeker",
                            checked: *role.read() == TeamRole::Seeker,
                            onchange: move |_| role.set(TeamRole::Seeker),
                        }
                        "Seeker"
                    }
                }

                if let Some(msg) = error.read().as_ref() {
                    p { class: "form-error", "{msg}" }
                }

                button {
                    r#type: "submit",
                    class: "btn btn--primary",
                    disabled: *loading.read(),
                    if *loading.read() { "Joining…" } else { "Join Game" }
                }
            }
        }
    }
}
