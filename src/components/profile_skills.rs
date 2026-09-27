use crate::{
    app::AuthVersion,
    client::{
        auth::get_current_user,
        profiles::{get_skills, set_skills},
    },
};
use dioxus::prelude::*;

#[component]
pub fn ProfileSkills(username: String) -> Element {
    let version = use_context::<AuthVersion>();
    let me = use_resource(move || {
        let _ = (version.0)();
        get_current_user()
    });
    let mut skills = use_resource(use_reactive((&username,), |(username,)| async move {
        get_skills(username).await
    }));
    let mut input = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let is_owner = me
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .and_then(|u| u.as_ref())
        .is_some_and(|u| u.username == username);
    let add = move |e: Event<FormData>| {
        e.prevent_default();
        if busy() {
            return;
        }
        let Some(Ok(mut next)) = skills() else {
            return;
        };
        next.push(input());
        busy.set(true);
        error.set(None);
        spawn(async move {
            match set_skills(next).await {
                Ok(_) => {
                    input.set(String::new());
                    skills.restart();
                }
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };
    rsx! {
        section { class: "skills-section",
            div { class: "section-heading", h2 { "Skills & expertise" } span { "The things you know best." } }
            match &*skills.read() {
                Some(Ok(items)) => rsx! {
                    div { class: "skill-tags",
                        for skill in items {
                            span { class: "skill-tag", key: "{skill}", "{skill}"
                                if is_owner {
                                    button { r#type: "button", disabled: busy(), "aria-label": "Remove {skill}",
                                        onclick: {
                                            let skill = skill.clone();
                                            move |_| {
                                                let Some(Ok(mut next)) = skills() else { return; };
                                                next.retain(|s| s != &skill);
                                                busy.set(true); error.set(None);
                                                spawn(async move { match set_skills(next).await { Ok(_) => skills.restart(), Err(e) => error.set(Some(e)) }; busy.set(false); });
                                            }
                                        }, "×"
                                    }
                                }
                            }
                        }
                        if items.is_empty() { p { class: "muted", "No skills added yet." } }
                    }
                    if is_owner {
                        form { class: "inline-form", onsubmit: add,
                            input { "aria-label": "New skill", placeholder: "e.g. Rust, product design, storytelling", maxlength: 40, required: true, value: "{input}", oninput: move |e| input.set(e.value()) }
                            button { class: "btn btn-primary", r#type: "submit", disabled: busy() || items.len() >= 12, "＋ Add skill" }
                        }
                        p { class: "field-hint", "Up to 12 skills. These appear on your public profile." }
                    }
                },
                Some(Err(e)) => rsx! { p { class: "error-state", role: "alert", "{e}" } },
                None => rsx! { p { class: "muted", "Loading skills…" } },
            }
            if let Some(e) = error() { p { class: "auth-error", role: "alert", "{e}" } }
        }
    }
}
