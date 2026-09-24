mod host_setup;
mod new_map;
mod edit_map_info;
mod edit_map_boundary;
mod edit_map_transit;
mod edit_map_confirm;
mod landing;
mod game;

pub use host_setup::HostSetup;
pub use new_map::NewMap;
pub use edit_map_info::EditMapInfo;
pub use edit_map_boundary::EditMapBoundary;
pub use edit_map_transit::EditMapTransit;
pub use edit_map_confirm::EditMapConfirm;
pub use landing::LandingPage;
pub use game::GameView;

use dioxus::prelude::*;

/// Wizard step progress indicator shared across all 4 map-creation steps.
#[component]
pub fn WizardSteps(current: u8) -> Element {
    let steps = [
        (1u8, "Info"),
        (2u8, "Boundary"),
        (3u8, "Transit"),
        (4u8, "Confirm"),
    ];
    rsx! {
        nav { class: "wizard-steps",
            for (i, (num, label)) in steps.iter().enumerate() {
                {
                    let num = *num;
                    let label = *label;
                    let cls = if num < current {
                        "wizard-step wizard-step--done"
                    } else if num == current {
                        "wizard-step wizard-step--active"
                    } else {
                        "wizard-step wizard-step--upcoming"
                    };
                    rsx! {
                        if i > 0 { div { class: "wizard-steps__connector" } }
                        div { class: cls,
                            span { class: "wizard-step__num", "{num}" }
                            span { class: "wizard-step__label", "{label}" }
                        }
                    }
                }
            }
        }
    }
}
