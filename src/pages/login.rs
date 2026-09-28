use crate::ui::{Button, ButtonType, Field, Input};
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

                       Field { id: "email", label: "Email",
    Input {
                               input_type: "email",
                               name: "email",
                               autocomplete: "username",
                               placeholder: "you@example.com",
                               required: true,
                               value: email(),
                               on_value_change: move |value| email.set(value),
    }
    }

                       Field { id: "password", label: "Password",
    Input {
                               input_type: "password",
                               name: "password",
                               autocomplete: "current-password",
                               placeholder: "••••••••",
                               required: true,
                               value: password(),
                               on_value_change: move |value| password.set(value),
    }
    }

                       Button {
                           button_type: ButtonType::Submit,
                           full_width: true,
                           loading: loading(),
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
