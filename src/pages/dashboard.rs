use dioxus::prelude::*;

use crate::client::auth::get_current_user;
use crate::client::posts::{delete_post, get_user_posts, publish_post, unpublish_post};
use crate::enums::app::Route;

#[component]
pub fn DashboardPage() -> Element {
    let auth_version = use_context::<crate::app::AuthVersion>();
    let user = use_resource(move || {
        let _version = (auth_version.0)();
        get_current_user()
    });
    let mut posts = use_resource(move || {
        let _version = (auth_version.0)();
        get_user_posts()
    });
    let mut action_error = use_signal(|| Option::<String>::None);
    let nav = use_navigator();
    let mut busy = use_signal(|| false);
    let mut pending_delete = use_signal(|| Option::<String>::None);

    // Redirect if not logged in
    let _user_data = match &*user.read() {
        Some(Ok(Some(u))) => Some(u.clone()),
        Some(Ok(None)) => {
            nav.push(Route::LoginPage {});
            return rsx! {};
        }
        _ => None,
    };

    let counts = posts
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(|items| {
            let published = items.iter().filter(|p| p.published_at.is_some()).count();
            (items.len(), published, items.len() - published)
        })
        .unwrap_or_default();

    rsx! {
        section {
            class: "dashboard-page",

            div {
                class: "dashboard-header",
                div { p { class: "eyebrow green", "YOUR WORKSPACE" } h1 { "My stories" } p { "A home for your drafts, stories, and next ideas." } }
                Link {
                    to: Route::WritePage {},
                    class: "btn btn-primary",
                    "＋ New draft"
                }
            }

            div { class: "stats-grid",
                div { class: "stat-card", p { class: "eyebrow", "ALL STORIES" } strong { "{counts.0}" } small { "Ideas in your collection" } }
                div { class: "stat-card", p { class: "eyebrow", "PUBLISHED" } strong { "{counts.1}" } small { "Out in the world" } }
                div { class: "stat-card", p { class: "eyebrow", "IN PROGRESS" } strong { "{counts.2}" } small { "Room to keep thinking" } }
            }
            if let Some(err) = action_error() {
                div {
                    class: "auth-error", role: "alert",
                    "{err}"
                }
            }

            match &*posts.read() {
                Some(Ok(user_posts)) => {
                    if user_posts.is_empty() {
                        rsx! {
                            div {
                                class: "empty-feed",
                                div { class: "empty-icon", "📝" }
                                h3 { "No stories yet" }
                                p { "Start writing your first story!" }
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                class: "dashboard-posts",

                                for post in user_posts.iter() {
                                    div {
                                        key: "{post.id}",
                                        class: "dashboard-post-card",

                                        div {
                                            class: "dashboard-post-info",

                                            h3 {
                                                class: "dashboard-post-title",
                                                "{post.title}"
                                            }

                                            p {
                                                class: "dashboard-post-excerpt",
                                                "{post.excerpt}"
                                            }

                                            div {
                                                class: "dashboard-post-meta",
                                                span {
                                                    class: if post.published_at.is_some() { "status-badge published" } else { "status-badge draft" },
                                                    if post.published_at.is_some() { "Published" } else { "Draft" }
                                                }
                                                span { class: "meta-separator", "·" }
                                                span { "{post.reading_time_minutes} min read" }
                                            }
                                        }

                                        div {
                                            class: "dashboard-post-actions",

                                            Link {
                                                to: Route::EditPage { id: post.id.clone() },
                                                class: "btn btn-small btn-secondary",
                                                "Edit"
                                            }

                                            {
                                                let post_id = post.id.clone();
                                                let is_published = post.published_at.is_some();
                                                rsx! {
                                                    if is_published {
                                                        button {
                                                            class: "btn btn-small btn-outline", disabled: busy(),
                                                            onclick: {
                                                                let pid = post_id.clone();
                                                                move |_| {
                                                                    if busy() { return; }
                                                                    busy.set(true);
                                                                    action_error.set(None);
                                                                    let pid = pid.clone();
                                                                    spawn(async move {
                                                                        match unpublish_post(pid).await {
                                                                            Ok(()) => { posts.restart(); }
                                                                            Err(e) => { action_error.set(Some(e.to_string())); }
                                                                        }
                                                                        busy.set(false);
                                                                    });
                                                                }
                                                            },
                                                            "Unpublish"
                                                        }
                                                    } else {
                                                        button {
                                                            class: "btn btn-small btn-success", disabled: busy(),
                                                            onclick: {
                                                                let pid = post_id.clone();
                                                                move |_| {
                                                                    if busy() { return; }
                                                                    busy.set(true);
                                                                    action_error.set(None);
                                                                    let pid = pid.clone();
                                                                    spawn(async move {
                                                                        match publish_post(pid).await {
                                                                            Ok(()) => { posts.restart(); }
                                                                            Err(e) => { action_error.set(Some(e.to_string())); }
                                                                        }
                                                                        busy.set(false);
                                                                    });
                                                                }
                                                            },
                                                            "Publish"
                                                        }
                                                    }
                                                }
                                            }

                                            if pending_delete().as_deref() == Some(post.id.as_str()) {
                                                span { class: "delete-warning", "Delete this story permanently?" }
                                                button { class: "btn btn-small btn-outline", disabled: busy(), onclick: move |_| pending_delete.set(None), "Keep story" }
                                                button {
                                                    class: "btn btn-small btn-danger", disabled: busy(),
                                                    onclick: {
                                                        let pid = post.id.clone();
                                                        move |_| {
                                                            if busy() { return; }
                                                            busy.set(true);
                                                            action_error.set(None);
                                                            let pid = pid.clone();
                                                            spawn(async move {
                                                                match delete_post(pid).await {
                                                                    Ok(()) => { pending_delete.set(None); posts.restart(); }
                                                                    Err(e) => action_error.set(Some(e)),
                                                                }
                                                                busy.set(false);
                                                            });
                                                        }
                                                    }, "Confirm delete"
                                                }
                                            } else {
                                                button { class: "btn btn-small btn-danger", disabled: busy(), onclick: { let pid = post.id.clone(); move |_| pending_delete.set(Some(pid.clone())) }, "Delete" }
                                            }

                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    div {
                        class: "error-state",
                        p { "Failed to load your stories: {e}" }
                    }
                },
                None => rsx! {
                    div {
                        class: "loading-state",
                        div { class: "spinner" }
                        p { "Loading your stories..." }
                    }
                },
            }
        }
    }
}
