use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct DialogProps {
    /// Unique and stable for the lifetime of this mounted component.
    pub id: String,
    pub open: ReadSignal<bool>,
    pub on_open_change: EventHandler<bool>,
    pub title: String,
    pub description: Option<String>,
    pub children: Element,
    #[props(default = true)]
    pub dismissible: bool,
    #[props(default)]
    pub close_on_backdrop: bool,
    #[props(default)]
    pub class: String,
    #[props(default = "Close dialog".to_string())]
    pub close_label: String,
}

/// Controlled browser modal using the native dialog top layer. The browser
/// supplies focus containment, inert background, and focus restoration.
/// Mount inside Theme. No portal means theme variables inherit naturally.
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let mut mounted = use_signal(|| false);
    let id = props.id.clone();
    use_effect(move || {
        let open = (props.open)();
        if mounted() {
            let eval = document::eval(
                r#"
                const [id, open] = await dioxus.recv();
                document.getElementById(id)?._fuiDialog?.setOpen(open);
            "#,
            );
            let _ = eval.send((&id, open));
        }
    });
    let mount_id = props.id.clone();
    rsx! {
        dialog {
            class: "fui-dialog {props.class}",
            id: props.id.clone(),
            "aria-labelledby": "{props.id}-title",
            "aria-describedby": props.description.as_ref().map(|_| format!("{}-description", props.id)),
            "data-open": (props.open)().to_string(),
            "data-dismissible": props.dismissible.to_string(),
            "data-backdrop-dismiss": props.close_on_backdrop.to_string(),
            onmounted: move |_| {
                let mut eval = document::eval(include_str!("dialog.js"));
                let _ = eval.send(&mount_id);
                spawn(async move {
                    while let Ok(message) = eval.recv::<String>().await {
                        if message == "ready" { mounted.set(true); }
                        if message == "dismiss" { props.on_open_change.call(false); }
                    }
                });
            },
            div { class: "fui-dialog-header",
                h2 { class: "fui-dialog-title", id: "{props.id}-title", "{props.title}" }
                button {
                    class: "fui-dialog-close", r#type: "button", autofocus: true,
                    disabled: !props.dismissible,
                    "aria-label": props.close_label,
                    onclick: move |_| { if props.dismissible { props.on_open_change.call(false); } },
                    "×"
                }
            }
            if let Some(ref description) = props.description {
                p { class: "fui-dialog-description", id: "{props.id}-description", "{description}" }
            }
            div { class: "fui-dialog-body", {props.children} }
        }
    }
}
