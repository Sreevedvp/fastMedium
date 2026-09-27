use dioxus::prelude::*;

use crate::client::auth;
use crate::enums::app::Route;

#[component]
pub fn RegisterPage() -> Element {
    let mut username = use_signal(|| String::new());
    let mut display_name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let nav = use_navigator();
    let mut auth_version = use_context::<crate::app::AuthVersion>();

    let handle_register = move |e: Event<FormData>| {
        e.prevent_default();
        let nav = nav.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            let password_val = password();
            let confirm_val = confirm_password();

            if password_val != confirm_val {
                error.set(Some("Passwords do not match".to_string()));
                loading.set(false);
                return;
            }

            let username_val = username();
            let email_val = email();
            let display_val = display_name();

            match auth::register(username_val, email_val, password_val, display_val).await {
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

                h1 { class: "auth-title", "Join fastMedium" }
                p { class: "auth-subtitle", "Create your account and start writing" }

                if let Some(err) = error() {
                    div {
                        class: "auth-error",
                        role: "alert",
                        "{err}"
                    }
                }

                form {
                    class: "auth-form",
                    onsubmit: handle_register,

                    div {
                        class: "form-row",

                        div {
                            class: "form-group",
                            label { r#for: "username", "Username" }
                            input {
                                r#type: "text",
                                id: "username",
                                placeholder: "johndoe",
                                required: true,
                                value: "{username}",
                                oninput: move |e| username.set(e.value()),
                            }
                        }

                        div {
                            class: "form-group",
                            label { r#for: "display_name", "Display Name" }
                            input {
                                r#type: "text",
                                id: "display_name",
                                placeholder: "John Doe",
                                required: true,
                                value: "{display_name}",
                                oninput: move |e| display_name.set(e.value()),
                            }
                        }
                    }

                    div {
                        class: "form-group",
                        label { r#for: "reg-email", "Email" }
                        input {
                            r#type: "email",
                            id: "reg-email",
                            name: "email",
                            autocomplete: "email",
                            placeholder: "you@example.com",
                            required: true,
                            value: "{email}",
                            oninput: move |e| email.set(e.value()),
                        }
                    }

                    div {
                        class: "form-row",

                        div {
                            class: "form-group",
                            label { r#for: "reg-password", "Password" }
                            input {
                                r#type: "password",
                                id: "reg-password",
                                autocomplete: "new-password",
                                placeholder: "Min 8 characters",
                                required: true,
                                value: "{password}",
                                oninput: move |e| password.set(e.value()),
                            }
                        }

                        div {
                            class: "form-group",
                            label { r#for: "confirm-password", "Confirm Password" }
                            input {
                                r#type: "password",
                                id: "confirm-password",
                                autocomplete: "new-password",
                                placeholder: "••••••••",
                                required: true,
                                value: "{confirm_password}",
                                oninput: move |e| confirm_password.set(e.value()),
                            }
                        }
                    }

                    button {
                        r#type: "submit",
                        class: "btn btn-primary btn-full",
                        disabled: loading(),
                        if loading() { "Creating account..." } else { "Create Account" }
                    }
                }

                p {
                    class: "auth-footer",
                    "Already have an account? "
                    Link {
                        to: Route::LoginPage {},
                        "Sign in"
                    }
                }
            }
        }
    }
}
