use dioxus::prelude::*;

/// Plain CSS: usable without Rust or Dioxus. No external fonts are fetched.
pub const FASTMEDIUM_CSS: &str = include_str!("theme.css");

/// Mount once per document. Components do not inject duplicate styles.
#[component]
pub fn ThemeStyles() -> Element {
    rsx! { document::Style { id: "fast-ui-theme", {FASTMEDIUM_CSS} } }
}

#[derive(Clone, PartialEq, Props)]
pub struct ThemeProps {
    pub children: Element,
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub compact: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A scoped theme. Override `--fui-*` variables on this element to customize it.
#[component]
pub fn Theme(props: ThemeProps) -> Element {
    // Explicit component props own these attributes; forwarded attributes cannot
    // bypass state or accessibility invariants.
    let attributes = props
        .attributes
        .into_iter()
        .filter(|attribute| !matches!(attribute.name, "class" | "data-fui-theme" | "data-density"))
        .collect::<Vec<_>>();
    rsx! {
        div {
            class: "fui-theme {props.class}",
            "data-fui-theme": "fastmedium",
            "data-density": if props.compact { "compact" } else { "comfortable" },
            ..attributes,
            {props.children}
        }
    }
}
