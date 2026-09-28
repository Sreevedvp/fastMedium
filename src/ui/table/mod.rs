use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct TableProps {
    /// A visible accessible name for the table and its scroll region.
    pub caption: String,
    pub children: Element,
    #[props(default)]
    pub class: String,
    #[props(extends = table)]
    pub attributes: Vec<Attribute>,
}

/// Semantic table, with a keyboard-scrollable overflow region. Supply native
/// thead/tbody/tr/th/td children; they retain the full HTML attribute API.
#[component]
pub fn Table(props: TableProps) -> Element {
    // Explicit component props own these attributes; forwarded attributes cannot
    // bypass state or accessibility invariants.
    let attributes = props
        .attributes
        .into_iter()
        .filter(|attribute| !matches!(attribute.name, "class"))
        .collect::<Vec<_>>();
    rsx! {
        div { class: "fui-table-scroll", role: "region", "aria-label": props.caption.clone(), tabindex: "0",
            table { class: "fui-table {props.class}", ..attributes,
                caption { "{props.caption}" }
                {props.children}
            }
        }
    }
}
