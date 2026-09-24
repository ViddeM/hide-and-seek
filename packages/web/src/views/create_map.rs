use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn CreateMap() -> Element {
    let nav = use_navigator();
    rsx! {
        ui::CreateMapWizard {
            on_created: move |_map| {
                let _ = nav.push(Route::HostSetup {});
            },
        }
    }
}
