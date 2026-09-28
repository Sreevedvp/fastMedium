use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Destructive,
}

impl ButtonVariant {
    fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ButtonSize {
    fn name(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ButtonType {
    #[default]
    Button,
    Submit,
    Reset,
}

impl ButtonType {
    fn name(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Submit => "submit",
            Self::Reset => "reset",
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    pub children: Element,
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub size: ButtonSize,
    #[props(default)]
    pub button_type: ButtonType,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub loading: bool,
    #[props(default)]
    pub full_width: bool,
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
}

/// An action button. Use a real anchor for navigation. Keep a visible label
/// while loading; loading prevents duplicate submissions.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    // Explicit component props own these attributes; forwarded attributes cannot
    // bypass state or accessibility invariants.
    let attributes = props
        .attributes
        .into_iter()
        .filter(|attribute| {
            !matches!(
                attribute.name,
                "class"
                    | "type"
                    | "disabled"
                    | "aria-busy"
                    | "data-variant"
                    | "data-size"
                    | "data-full-width"
                    | "onclick"
            )
        })
        .collect::<Vec<_>>();
    let disabled = props.disabled || props.loading;
    rsx! {
        button {
            class: "fui-button {props.class}",
            r#type: props.button_type.name(),
            disabled: disabled,
            "data-variant": props.variant.name(),
            "data-size": props.size.name(),
            "data-full-width": props.full_width.to_string(),
            "aria-busy": props.loading.to_string(),
            onclick: move |event| { if !disabled { props.onclick.call(event); } },
            ..attributes,
            if props.loading { span { class: "fui-spinner", "aria-hidden": "true" } }
            {props.children}
        }
    }
}
