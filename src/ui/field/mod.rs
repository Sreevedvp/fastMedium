use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct FieldContext {
    id: ReadSignal<String>,
    hint: ReadSignal<Option<String>>,
    error: ReadSignal<Option<String>>,
}

#[derive(Clone, PartialEq, Props)]
pub struct FieldProps {
    /// Unique, stable ID shared with the control and its label.
    pub id: ReadSignal<String>,
    pub label: String,
    pub children: Element,
    #[props(default)]
    pub hint: ReadSignal<Option<String>>,
    #[props(default)]
    pub error: ReadSignal<Option<String>>,
    #[props(default)]
    pub class: String,
}

/// Wrap one Input. Native/custom controls may also be children, but must wire
/// their own ID, aria-invalid, and described-by attributes.
#[component]
pub fn Field(props: FieldProps) -> Element {
    use_context_provider(|| FieldContext {
        id: props.id,
        hint: props.hint,
        error: props.error,
    });
    let id = (props.id)();
    rsx! {
        div { class: "fui-field {props.class}",
            label { class: "fui-label", r#for: id.clone(), "{props.label}" }
            {props.children}
            if let Some(hint) = (props.hint)() {
                p { class: "fui-hint", id: "{id}-hint", "{hint}" }
            }
            if let Some(error) = (props.error)() {
                p { class: "fui-error", id: "{id}-error", role: "alert", "{error}" }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct InputProps {
    pub value: ReadSignal<String>,
    pub on_value_change: EventHandler<String>,
    /// Defaults to the enclosing Field's ID. A supplied ID must match its Field.
    pub id: Option<String>,
    #[props(default = "text".to_string())]
    pub input_type: String,
    #[props(default)]
    pub class: String,
    /// Additional description IDs, combined with the Field hint and error.
    #[props(default)]
    pub described_by: String,
    #[props(default)]
    pub invalid: bool,
    #[props(extends = input)]
    pub attributes: Vec<Attribute>,
}

/// Controlled text-like input. Native attributes (name, autocomplete, required,
/// disabled, readonly, min/max, etc.) are forwarded to the underlying input.
#[component]
pub fn Input(props: InputProps) -> Element {
    // Explicit component props own these attributes; forwarded attributes cannot
    // bypass state or accessibility invariants.
    let attributes = props
        .attributes
        .into_iter()
        .filter(|attribute| {
            !matches!(
                attribute.name,
                "id" | "class" | "type" | "value" | "aria-invalid" | "aria-describedby" | "oninput"
            )
        })
        .collect::<Vec<_>>();
    let field = try_use_context::<FieldContext>();
    let id = props.id.or_else(|| field.map(|f| (f.id)()));
    let mut descriptions = vec![props.described_by];
    let mut invalid = props.invalid;
    if let Some(field) = field {
        let field_id = (field.id)();
        if (field.hint)().is_some() {
            descriptions.push(format!("{field_id}-hint"));
        }
        if (field.error)().is_some() {
            descriptions.push(format!("{field_id}-error"));
            invalid = true;
        }
    }
    let described_by = descriptions
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    rsx! {
        input {
            id: id,
            class: "fui-input {props.class}",
            r#type: props.input_type,
            value: (props.value)(),
            "aria-invalid": invalid.to_string(),
            "aria-describedby": (!described_by.is_empty()).then_some(described_by),
            oninput: move |event| props.on_value_change.call(event.value()),
            ..attributes,
        }
    }
}
