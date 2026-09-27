use dioxus::prelude::*;

use crate::enums::app::Route;

pub const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Clone, Copy)]
pub struct AuthVersion(pub Signal<u64>);

#[derive(Clone, Copy)]
pub struct FeedSearch(pub Signal<String>);

#[component]
pub fn App() -> Element {
    use_context_provider(|| AuthVersion(Signal::new(0)));
    use_context_provider(|| FeedSearch(Signal::new(String::new())));
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: MAIN_CSS,
        }

        Router::<Route> {}
    }
}
