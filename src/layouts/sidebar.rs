use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside {
            class: "sidebar",

            h2 { "Explore" }

            ul {
                li { "For you" }
                li { "Following" }
                li { "Technology" }
                li { "Design" }
            }
        }
    }
}
