use dioxus::prelude::*;

use crate::client::auth;
use crate::enums::app::Route;

#[component]
pub fn LoginPage() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let nav = use_navigator();
    let mut auth_version = use_context::<crate::app::AuthVersion>();

    let handle_login = move |e: Event<FormData>| {
        e.prevent_default();
        let nav = nav.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            let email_val = email();
            let password_val = password();

            match auth::login(email_val, password_val).await {
                Ok(_user) => {
                    auth_version.0 += 1;
                    nav.push(Route::HomePage {});
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        section {
            class: "auth-page",

            div {
                class: "auth-card",
                p { class: "eyebrow", "YOUR NEXT CHAPTER" }

                h1 { class: "auth-title", "Welcome back" }
                p { class: "auth-subtitle", "Sign in to your account" }

                if let Some(err) = error() {
                    div {
                        class: "auth-error",
                        role: "alert",
                        "{err}"
                    }
                }

                form {
                    class: "auth-form",
                    onsubmit: handle_login,

                    div {
                        class: "form-group",
                        label { r#for: "email", "Email" }
                        input {
                            r#type: "email",
                            id: "email",
                            name: "email",
                            autocomplete: "username",
                            placeholder: "you@example.com",
                            required: true,
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                        }
                    }

                    div {
                        class: "form-group",
                        label { r#for: "password", "Password" }
                        input {
                            r#type: "password",
                            id: "password",
                            name: "password",
                            autocomplete: "current-password",
                            placeholder: "••••••••",
                            required: true,
                            value: "{password}",
                            oninput: move |e| password.set(e.value()),
                        }
                    }

                    button {
                        r#type: "submit",
                        class: "btn btn-primary btn-full",
                        disabled: loading(),
                        if loading() { "Signing in..." } else { "Sign In" }
                    }
                }

                p {
                    class: "auth-footer",
                    "Don't have an account? "
                    Link {
                        to: Route::RegisterPage {},
                        "Create one"
                    }
                }
            }
        }
    }
}
