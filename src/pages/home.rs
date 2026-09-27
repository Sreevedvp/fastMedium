use crate::app::FeedSearch;
use crate::client::posts::get_published_posts;
use crate::components::post_card::PostCard;
use crate::enums::app::Route;
use dioxus::prelude::*;

pub const COVER: Asset = asset!("/assets/editorial-cover.jpg");

#[component]
pub fn HomePage() -> Element {
    let search = use_context::<FeedSearch>();
    let mut page = use_signal(|| 1u32);
    let mut posts = use_resource(move || get_published_posts(page(), 20, (search.0)()));
    use_effect(move || {
        let _query = (search.0)();
        page.set(1);
    });
    let query = (search.0)().trim().to_lowercase();
    rsx! {
        section { class: "home-page",
            header { class: "feed-welcome",
                p { class: "eyebrow", "FOR THE CURIOUS & THE CREATIVE" }
                h1 { "A space for " em { "ideas" } br {} "you’ll love to " em { "come back to." } }
                p { "Share what you know. Find your people. Make something worth reading." }
                div { class: "welcome-actions",
                    Link { to: Route::WritePage {}, class: "btn btn-primary", "✎  Write your story" }
                    Link { to: Route::PublicationsPage {}, class: "underlined-link", "Explore publications ↗" }
                }
            }
            div { class: "context-stripe",
                span { span { class: "live-dot" } "An independent space for ideas" }
                span { class: "mono", "READ. REFLECT. CONNECT." }
            }
            div { class: "publication-grid",
                div { class: "feed-main",
                    div { class: "feed-toolbar",
                        div { class: "feed-tabs", span { class: "selected", "Latest stories" } Link { to: Route::PublicationsPage {}, "Publications ↗" } }
                        span { class: "muted", "New perspectives, at your pace" }
                    }
                    match &*posts.read() {
                        Some(Ok(items)) => {
                            let filtered: Vec<_> = items.iter().collect();
                            rsx! {
                                if items.is_empty() && query.is_empty() && page() == 1 {

                                    div { class: "empty-publication",
                                        span { class: "eyebrow", "THE NEXT STORY COULD BE YOURS" }
                                        h2 { "Every publication starts with an idea." }
                                        p { "No stories have been published yet. Open a fresh page and make this space your own." }
                                        Link { to: Route::WritePage {}, class: "btn btn-primary", "Write the first story ↗" }
                                    }
                                } else if filtered.is_empty() {
                                    div { class: "empty-feed", h2 { "No stories found" } p { "Try another word or return to All stories." } }
                                } else {
                                    div { class: "post-feed",
                                        for (index, post) in filtered.iter().enumerate() {
                                            div { class: if index == 0 { "lead-story" } else { "feed-story" },
                                                if index == 0 { p { class: "eyebrow green", "LATEST DISPATCH" } }
                                                PostCard { key: "{post.id}", id: post.id.clone(), title: post.title.clone(), slug: post.slug.clone(), excerpt: post.excerpt.clone(), author_username: post.author_username.clone(), author_display_name: post.author_display_name.clone(), published_at: post.published_at.clone(), reading_time_minutes: post.reading_time_minutes }
                                            }
                                        }
                                    }
                                }
                                if page() > 1 || items.len() == 20 {
                                    div { class: "pagination",
                                        button { class: "btn btn-outline", disabled: page() == 1, onclick: move |_| page -= 1, "← Newer stories" }
                                        span { "Page {page}" }
                                        button { class: "btn btn-outline", disabled: items.len() < 20, onclick: move |_| page += 1, "Older stories →" }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { div { class: "error-state", role: "alert", "Could not load stories: {e}" button { class: "btn btn-outline", onclick: move |_| posts.restart(), "Try again" } } },
                        None => rsx! { div { class: "loading-state", "Gathering the latest ideas…" } },
                    }
                }
                PublicationRail {}
            }
        }
    }
}

#[component]
fn PublicationRail() -> Element {
    let publications = use_resource(crate::client::publications::list);
    rsx! {
        aside { class: "publication-rail",
            section { class: "rail-card",
                p { class: "eyebrow", "FIND YOUR CORNER" }
                h3 { "Small publications.\nBig possibilities." }
                match &*publications.read() {
                    Some(Ok(items)) => rsx! {
                        for publication in items.iter().take(4) {
                            Link { to: Route::PublicationPage { slug: publication.slug.clone() }, class: "rail-publication", strong { "{publication.name}" } span { "{publication.story_count} stories ↗" } }
                        }
                        if items.is_empty() { p { "A favourite subject deserves a space of its own. Create the first publication." } }
                    },
                    Some(Err(_)) => rsx! { p { "Browse publications to find your next read." } },
                    None => rsx! { p { "Loading publications…" } },
                }
                Link { to: Route::PublicationsPage {}, class: "underlined-link", "All publications ↗" }
            }
            section { class: "rail-card notebook-card",
                span { class: "big-asterisk", "✳" }
                h3 { "You know something\nworth sharing." }
                p { "A skill, an observation, a new perspective. Give it a little room to grow." }
                Link { to: Route::WritePage {}, class: "underlined-link", "Start a story ↗" }
            }
        }
    }
}
