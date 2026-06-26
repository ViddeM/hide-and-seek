use dioxus::prelude::*;

#[component]
pub fn GameCode(code: String) -> Element {
    let mut copied = use_signal(|| false);

    let code_clone = code.clone();
    let copy = move |_| {
        let js = format!(
            "navigator.clipboard.writeText('{}').catch(()=>{{}});\
             setTimeout(()=>window._hideseekCopiedReset&&window._hideseekCopiedReset(),2000);",
            code_clone
        );
        let _ = document::eval(&js);
        copied.set(true);
    };

    rsx! {
        div {
            class: "game-code",
            span { class: "game-code__label", "Game code" }
            button {
                class: "game-code__value",
                onclick: copy,
                title: "Click to copy",
                "{code}"
            }
            if *copied.read() {
                span { class: "game-code__copied", "Copied!" }
            }
        }
    }
}
