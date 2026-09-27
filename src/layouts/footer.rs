use dioxus::prelude::*;
#[component]
pub fn Footer() -> Element {
    rsx! { footer { class: "site-footer",
        span { "fastMedium" }
        p { "Creating little spaces for big ideas." }
        small { "© 2026 fastMedium" }
    } }
}
