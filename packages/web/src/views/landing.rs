use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn LandingPage() -> Element {
    let nav = use_navigator();
    rsx! {
        ui::landing::Landing {
            on_join: move |code: String| {
                let _ = document::eval(&format!(
                    "sessionStorage.setItem('joinCode','{code}')"
                ));
                let _ = nav.push(Route::JoinGame {});
            },
            on_host: move |_: ()| { let _ = nav.push(Route::HostSetup {}); },
        }
    }
}
