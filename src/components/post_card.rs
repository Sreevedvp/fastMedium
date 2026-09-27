use dioxus::prelude::*;

use crate::enums::app::Route;

#[component]
pub fn PostCard(
    id: String,
    title: String,
    slug: String,
    excerpt: String,
    author_username: String,
    author_display_name: String,
    published_at: Option<String>,
    reading_time_minutes: u32,
) -> Element {
    rsx! {
        div {
            class: "post-card",

            Link {
                to: Route::PostPage { slug: slug.clone() },
                class: "post-card-link",

                h3 { class: "post-card-title", "{title}" }

                if !excerpt.is_empty() {
                    p { class: "post-card-excerpt", "{excerpt}" }
                }
            }

            div {
                class: "post-card-meta",

                Link {
                    to: Route::ProfilePage { username: author_username.clone() },
                    class: "post-card-author",

                    div {
                        class: "author-avatar-small",
                        "{author_display_name.chars().next().unwrap_or('?')}"
                    }
                    span { "{author_display_name}" }
                }

                if let Some(ref date) = published_at {
                    span { class: "meta-separator", "·" }
                    span { class: "post-card-date", "{date}" }
                }

                span { class: "meta-separator", "·" }
                span { class: "post-card-reading-time", "{reading_time_minutes} min read" }
            }
        }
    }
}
