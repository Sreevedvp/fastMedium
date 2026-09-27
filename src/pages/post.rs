use dioxus::prelude::*;

use crate::client::posts::get_post_by_slug;
use crate::enums::app::Route;

#[component]
pub fn PostPage(slug: String) -> Element {
    let post = use_resource(use_reactive((&slug,), |(slug,)| get_post_by_slug(slug)));

    rsx! {
        section {
            class: "post-page",

            match &*post.read() {
                Some(Ok(Some(post))) => rsx! {
                    article {
                        class: "article",

                        header {
                            class: "article-header",

                            h1 { class: "article-title", "{post.title}" }

                            div {
                                class: "article-meta",

                                Link {
                                    to: Route::ProfilePage { username: post.author_username.clone() },
                                    class: "author-link",

                                    div {
                                        class: "author-avatar",
                                        "{post.author_display_name.chars().next().unwrap_or('?')}"
                                    }
                                    span { class: "author-name", "{post.author_display_name}" }
                                }

                                if let Some(ref pub_date) = post.published_at {
                                    span { class: "meta-separator", "·" }
                                    span { class: "publish-date", "{pub_date}" }
                                }

                                span { class: "meta-separator", "·" }
                                span { class: "reading-time", "{post.reading_time_minutes} min read" }
                            }

                            if !post.excerpt.is_empty() {
                                p { class: "article-excerpt", "{post.excerpt}" }
                            }
                        }

                        div {
                            class: "article-content prose",
                            dangerous_inner_html: "{post.content_html}",
                        }
                    }
                },
                Some(Ok(None)) => rsx! {
                    div {
                        class: "not-found-state",
                        h2 { "Story not found" }
                        p { "This story doesn't exist or has been removed." }
                        Link {
                            to: Route::HomePage {},
                            class: "btn btn-secondary",
                            "Back to Home"
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    div {
                        class: "error-state",
                        p { "Failed to load story: {e}" }
                    }
                },
                None => rsx! {
                    div {
                        class: "loading-state",
                        div { class: "spinner" }
                        p { "Loading story..." }
                    }
                },
            }
        }
    }
}
