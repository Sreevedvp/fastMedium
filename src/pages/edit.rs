use dioxus::prelude::*;

use crate::client::posts::{get_post_for_edit, update_post};
use crate::enums::app::Route;

#[component]
pub fn EditPage(id: String) -> Element {
    let mut post_data = use_resource(use_reactive((&id,), |(id,)| get_post_for_edit(id)));
    rsx! {
        match &*post_data.read() {
            Some(Ok(post)) => rsx! { EditForm { key: "{post.id}", post: post.clone() } },
            Some(Err(error)) => rsx! { section { class: "error-state", role: "alert", p { "Could not load story: {error}" } button { class: "btn btn-outline", onclick: move |_| post_data.restart(), "Try again" } Link { to: Route::DashboardPage {}, "Back to My stories" } } },
            None => rsx! { p { class: "loading-state", "Loading story…" } },
        }
    }
}

#[component]
fn EditForm(post: crate::models::post::Post) -> Element {
    let id = post.id.clone();
    let version = post.version;
    let mut title = use_signal(|| post.title.clone());
    let mut excerpt = use_signal(|| post.excerpt.clone());
    let mut content = use_signal(|| post.content.clone());
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let nav = use_navigator();

    let handle_save = move |_| {
        if loading() {
            return;
        }
        loading.set(true);
        let nav = nav.clone();
        let post_id = id.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            let title_val = title();
            let excerpt_val = excerpt();
            let content_val = content();

            match update_post(post_id, title_val, excerpt_val, content_val, version).await {
                Ok(()) => {
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
                        h1 { "Shape your story" }
                        div {
                            class: "editor-actions",
                            button {
                                class: "btn btn-primary",
                                disabled: loading(),
                                onclick: handle_save,
                                if loading() { "Saving..." } else { "Save Changes" }
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
                    }

            aside { class: "editor-inspector",
                p { class: "eyebrow green", "THE WRITING DESK" }
                h2 { "Story inspector" }
                div { class: "inspector-row", span { "Words" } strong { "{content().split_whitespace().count()}" } }
                div { class: "inspector-row", span { "Reading time" } strong { "{(content().split_whitespace().count().max(1) + 199) / 200} min" } }
                div { class: "inspector-row", span { "Format" } strong { "Markdown" } }
                p { "A few tools for a clearer story" }
                ul { li { code { "##" } " A section heading" } li { code { "**text**" } " Bold emphasis" } li { code { ">" } " A pull quote" } li { code { "-" } " A list of ideas" } }
                p { "Drafts stay private until published. Saves check for edits from other tabs; if there is a conflict, copy your text before reloading." }
                Link { to: Route::DashboardPage {}, class: "rail-link", "← My stories" }
            }
        }
    }
}
