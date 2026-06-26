use api::models::{CardType, DrawnCard};
use dioxus::prelude::*;

#[component]
pub fn CardDisplay(card: DrawnCard, on_play: Option<EventHandler<()>>) -> Element {
    let type_class = match card.card.card_type {
        CardType::Bonus => "card--bonus",
        CardType::Curse => "card--curse",
    };
    let type_label = match card.card.card_type {
        CardType::Bonus => "Bonus",
        CardType::Curse => "Curse",
    };

    rsx! {
        div {
            class: "card {type_class}",
            div {
                class: "card__header",
                span { class: "card__name", "{card.card.name}" }
                span { class: "card__type-badge", "{type_label}" }
            }
            p { class: "card__effect", "{card.card.effect}" }
            if let Some(flavor) = &card.card.flavor_text {
                em { class: "card__flavor", "{flavor}" }
            }
            if let Some(handler) = on_play {
                button {
                    class: "card__play-btn",
                    onclick: move |_| handler.call(()),
                    "Play"
                }
            }
        }
    }
}
