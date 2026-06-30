use dioxus::CapturedError;
use dioxus::prelude::*;

#[component]
pub fn SeekerView(game_id: String) -> Element {
    let data = use_resource(move || {
        let _gid = game_id.clone();
        async move {
            let session = api::auth::get_session()
                .await?
                .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
            let state = api::game::get_game_state(session.game_id).await?;
            let map = api::maps::get_map(state.map_id).await?;
            Ok::<_, CapturedError>((session, map))
        }
    });

    let (session, map) = (*data.suspend()?.read()).clone()?;

    rsx! {
        ui::seeker_view::SeekerViewComponent {
            game_id: session.game_id,
            session: session.clone(),
            map: map.clone(),
        }
    }
}
