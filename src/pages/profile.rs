use dioxus::prelude::*;

use crate::client::profiles::{get_author_posts, get_profile};
use crate::components::post_card::PostCard;
use crate::components::profile_skills::ProfileSkills;

#[component]
pub fn ProfilePage(username: String) -> Element {
    let profile = use_resource(use_reactive((&username,), |(username,)| async move {
        get_profile(username).await
    }));
    let posts = use_resource(use_reactive((&username,), |(username,)| async move {
        get_author_posts(username, 1).await
    }));

    rsx! {
        section {
            class: "profile-page",

            match &*profile.read() {
                Some(Ok(Some(user))) => rsx! {
                    div {
                        class: "profile-header",

                        div {
                            class: "profile-avatar-large",
                            "{user.display_name.chars().next().unwrap_or('?')}"
                        }

                        h1 { class: "profile-name", "{user.display_name}" }
                        p { class: "profile-username", "@{user.username}" }

                        if !user.bio.is_empty() {
                            p { class: "profile-bio", "{user.bio}" }
                        }

                        p {
                            class: "profile-joined",
                            "Joined {user.created_at}"
                        }
                    }

                    ProfileSkills { username: username.clone() }

                    div {
                        class: "profile-posts",

                        h2 { class: "section-title", "Published Stories" }

                        match &*posts.read() {
                            Some(Ok(author_posts)) => {
                                if author_posts.is_empty() {
                                    rsx! {
                                        div {
                                            class: "empty-feed",
                                            p { "No published stories yet." }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div {
                                            class: "post-feed",
                                            for post in author_posts.iter() {
                                                PostCard {
                                                    key: "{post.id}",
                                                    id: post.id.clone(),
                                                    title: post.title.clone(),
                                                    slug: post.slug.clone(),
                                                    excerpt: post.excerpt.clone(),
                                                    author_username: post.author_username.clone(),
                                                    author_display_name: post.author_display_name.clone(),
                                                    published_at: post.published_at.clone(),
                                                    reading_time_minutes: post.reading_time_minutes,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => rsx! {
                                div { class: "error-state", p { "Failed to load posts: {e}" } }
                            },
                            None => rsx! {
                                div { class: "loading-state", div { class: "spinner" } }
                            },
                        }
                    }
                },
                Some(Ok(None)) => rsx! {
                    div {
                        class: "not-found-state",
                        h2 { "Author not found" }
                        p { "This profile doesn't exist." }
                    }
                },
                Some(Err(e)) => rsx! {
                    div { class: "error-state", p { "Failed to load profile: {e}" } }
                },
                None => rsx! {
                    div {
                        class: "loading-state",
                        div { class: "spinner" }
                        p { "Loading profile..." }
                    }
                },
            }
        }
    }
}
