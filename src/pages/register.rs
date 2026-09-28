use crate::ui::{Button, ButtonType, Field, Input};
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

                           Field { id: "username", label: "Username",
    Input {
                                   input_type: "text",
                                   placeholder: "johndoe",
                                   required: true,
                                   value: username(),
                                   on_value_change: move |value| username.set(value),
    }
    }

                           Field { id: "display_name", label: "Display Name",
    Input {
                                   input_type: "text",
                                   placeholder: "John Doe",
                                   required: true,
                                   value: display_name(),
                                   on_value_change: move |value| display_name.set(value),
    }
    }
                       }

                       Field { id: "reg-email", label: "Email",
    Input {
                               input_type: "email",
                               name: "email",
                               autocomplete: "email",
                               placeholder: "you@example.com",
                               required: true,
                               value: email(),
                               on_value_change: move |value| email.set(value),
    }
    }

                       div {
                           class: "form-row",

                           Field { id: "reg-password", label: "Password",
    Input {
                                   input_type: "password",
                                   autocomplete: "new-password",
                                   placeholder: "Min 8 characters",
                                   required: true,
                                   value: password(),
                                   on_value_change: move |value| password.set(value),
    }
    }

                           Field { id: "confirm-password", label: "Confirm Password",
    Input {
                                   input_type: "password",
                                   autocomplete: "new-password",
                                   placeholder: "••••••••",
                                   required: true,
                                   value: confirm_password(),
                                   on_value_change: move |value| confirm_password.set(value),
    }
    }
                       }

                       Button {
                           button_type: ButtonType::Submit,
                           full_width: true,
                           loading: loading(),
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
