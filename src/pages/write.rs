use dioxus::prelude::*;

use crate::client::auth::get_current_user;
use crate::client::posts::create_post;
use crate::enums::app::Route;

#[component]
pub fn WritePage() -> Element {
    let auth_version = use_context::<crate::app::AuthVersion>();
    let user = use_resource(move || {
        let _version = (auth_version.0)();
        get_current_user()
    });
    let mut title = use_signal(|| String::new());
    let mut excerpt = use_signal(|| String::new());
    let mut content = use_signal(|| String::new());
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let nav = use_navigator();

    // Redirect if not logged in
    match &*user.read() {
        Some(Ok(Some(_))) => {}
        Some(Ok(None)) => {
            nav.push(Route::LoginPage {});
            return rsx! {};
        }
        _ => {}
    }

    let handle_save = move |_| {
        if loading() {
            return;
        }
        loading.set(true);
        let nav = nav.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            let title_val = title();
            let excerpt_val = excerpt();
            let content_val = content();

            match create_post(title_val, excerpt_val, content_val).await {
                Ok(_post) => {
                    nav.push(Route::DashboardPage {});
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        section {
            class: "editor-page",

            div {
                class: "editor-header",
                h1 { "A space to think" }
                div {
                    class: "editor-actions",
                    button {
                        class: "btn btn-primary",
                        disabled: loading() || !matches!(&*user.read(), Some(Ok(Some(_)))),
                        onclick: handle_save,
                        if loading() { "Saving..." } else { "Save Draft" }
                    }
                }
            }

            if let Some(err) = error() {
                div {
                    class: "auth-error", role: "alert",
                    "{err}"
                }
            }

            div {
                class: "editor-form",

                div {
                    class: "form-group",
                    input {
                        r#type: "text",
                        class: "editor-title-input", maxlength: 200, disabled: loading(),
                        "aria-label": "Story title",
                        placeholder: "Story title...",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                    }
                }

                div {
                    class: "form-group",
                    input {
                        r#type: "text",
                        class: "editor-excerpt-input", maxlength: 500, disabled: loading(),
                        "aria-label": "Story excerpt",
                        placeholder: "Write a brief excerpt...",
                        value: "{excerpt}",
                        oninput: move |e| excerpt.set(e.value()),
                    }
                }

                div {
                    class: "form-group",
                    textarea {
                        class: "editor-content-input", disabled: loading(),
                        "aria-label": "Story content",
                        placeholder: "Tell your story... (Markdown supported)",
                        rows: 20,
                        value: "{content}",
                        oninput: move |e| content.set(e.value()),
                    }
                }

                p {
                    class: "editor-hint",
                    "Write in Markdown. Save a draft now; publish when you’re ready."
                }
            }
            aside { class: "editor-inspector",
                p { class: "eyebrow green", "THE WRITING DESK" }
                h2 { "Story inspector" }
                div { class: "inspector-row", span { "Words" } strong { "{content().split_whitespace().count()}" } }
                div { class: "inspector-row", span { "Reading time" } strong { "{(content().split_whitespace().count().max(1) + 199) / 200} min" } }
                div { class: "inspector-row", span { "Format" } strong { "Markdown" } }
                p { "A few tools for a clearer story" }
                ul { li { code { "##" } " A section heading" } li { code { "**text**" } " Bold emphasis" } li { code { ">" } " A pull quote" } li { code { "-" } " A list of ideas" } }
                p { "Drafts stay private until you publish them from your publication dashboard." }
                Link { to: Route::DashboardPage {}, class: "rail-link", "← My stories" }
            }
        }
    }
}
