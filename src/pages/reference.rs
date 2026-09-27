use super::home::COVER;
use crate::enums::app::Route;
use dioxus::prelude::*;

#[component]
pub fn ReferencePage() -> Element {
    rsx! {
        section { class: "reader-page",
            div { class: "reader-breadcrumb", Link { to: Route::HomePage {}, "Publication" } span { "/" } span { "Systems & mind" } span { class: "pill green-pill", "Design preview" } }
            header { class: "reader-hero",
                p { class: "eyebrow green", "THE READING ROOM · SAMPLE ESSAY" }
                h1 { "The Architecture of Thought" }
                p { class: "reader-deck", "Granular knowledge in the age of synthesis. A meditation on the spaces between our ideas." }
                div { class: "article-meta", span { class: "author-avatar", "f" } div { strong { "The fastMedium reading room" } p { "Adapted from your design reference · 3 min read" } } }
            }
            div { class: "reader-grid",
                article { class: "prose reference-prose",
                    img { class: "reader-image", src: COVER, alt: "Sculptural glass blocks arranged on a warm paper surface" }
                    p { class: "image-credit", "A study in structure, space, and the connections between things." }
                    section { id: "thought",
                        p { class: "drop-cap", "For decades, we have imagined knowledge as a collection of documents: folders inside folders, each idea assigned a single place. Yet thinking has never worked that way. One observation recalls another. A question in one field opens a door in another." }
                        p { "A useful personal library makes room for that movement. Rather than asking only where an idea belongs, we can ask what it connects to—and what those connections make possible." }
                    }
                    section { id: "connections", h2 { "From documents to connections" }
                        p { "The smallest useful unit of knowledge might be a paragraph, a sketch, a question, or a passage you return to. Each is complete enough to stand on its own, but open enough to become part of something larger." }
                        blockquote { "A living collection of ideas is less like a filing cabinet and more like a garden." }
                        details { open: true,
                            summary { "What makes an idea worth connecting?" }
                            p { "Keep a note when it changes how you see something. Add context in your own words. Link it to another thought that it supports, questions, or extends." }
                        }
                    }
                    div { class: "editorial-callout", span { "↳" } p { strong { "Try this." } " Take two notes that seem unrelated. Write one sentence describing what they might have in common." } }
                    section { id: "practice", h2 { "A quieter writing practice" }
                        p { "Collecting is only the beginning. The value arrives when we revisit an observation, sharpen a question, and give an unfinished thought enough attention to become a story." }
                        table { thead { tr { th { "Start with" } th { "Make room for" } } } tbody {
                            tr { td { "An observation" } td { "A question worth asking" } }
                            tr { td { "A saved passage" } td { "Your own interpretation" } }
                            tr { td { "A rough draft" } td { "A new connection" } }
                        } }
                        p { "The aim is not to build a perfect system. It is to make returning to your ideas a little easier—and to leave enough space for the next unexpected connection." }
                    }
                    div { class: "reader-end", p { class: "eyebrow", "KEEP THE THOUGHT GOING" } h3 { "What will you connect next?" } Link { to: Route::WritePage {}, class: "btn btn-primary", "Start your own story ↗" } }
                }
                aside { class: "reader-outline",
                    p { class: "eyebrow", "IN THIS ESSAY" }
                    a { href: "#thought", "01  The shape of thought" }
                    a { href: "#connections", "02  Documents to connections" }
                    a { href: "#practice", "03  A quieter practice" }
                    div { class: "margin-note", p { class: "eyebrow", "A NOTE IN THE MARGIN" } p { "Leave a little space for an idea you didn’t expect." } }
                    Link { to: Route::HomePage {}, "← Back to the publication" }
                }
            }
        }
    }
}
