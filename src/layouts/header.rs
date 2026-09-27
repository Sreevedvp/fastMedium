use crate::app::{AuthVersion, FeedSearch};
use crate::client::auth::{get_current_user, logout};
use crate::enums::app::Route;
use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    let mut auth_version = use_context::<AuthVersion>();
    let mut search = use_context::<FeedSearch>();
    let user = use_resource(move || {
        let _version = (auth_version.0)();
        get_current_user()
    });
    let nav = use_navigator();
    let mut logout_error = use_signal(|| None::<String>);
    let mut signing_out = use_signal(|| false);
    let handle_logout = move |_| {
        spawn(async move {
            signing_out.set(true);
            logout_error.set(None);
            match logout().await {
                Ok(()) => {
                    auth_version.0 += 1;
                    nav.replace(Route::HomePage {});
                }
                Err(e) => logout_error.set(Some(e.to_string())),
            }
            signing_out.set(false);
        });
    };
    rsx! {
        header { class: "site-header",
            div { class: "header-inner",
                Link { to: Route::HomePage {}, class: "brand",
                    span { class: "brand-mark", "fM" }
                    span { class: "brand-lockup",
                        strong { "fastMedium" }
                        small { "A SPACE FOR YOUR IDEAS" }
                    }
                }
                form { class: "header-search", role: "search",
                    onsubmit: move |e| { e.prevent_default(); nav.push(Route::HomePage {}); },
                    span { class: "search-icon", "⌕" }
                    input { r#type: "search", "aria-label": "Search stories",
                        placeholder: "Search stories and ideas…", value: "{search.0}",
                        oninput: move |e| { search.0.set(e.value()); nav.push(Route::HomePage {}); }
                    }
                    kbd { "↵" }
                }
                nav { class: "header-nav", "aria-label": "Main navigation",
                    Link { to: Route::HomePage {}, class: "nav-link", "⌂  All stories" }
                    Link { to: Route::PublicationsPage {}, class: "nav-link", "▤  Publications" }
                    match &*user.read() {
                        Some(Ok(Some(current_user))) => rsx! {
                            Link { to: Route::DashboardPage {}, class: "nav-link", "▧  My stories" }
                            Link { to: Route::WritePage {}, class: "btn btn-primary btn-small", "＋  Write a story" }
                            Link { to: Route::ProfilePage { username: current_user.username.clone() },
                                class: "nav-link profile-nav", "aria-label": "Skills and profile",
                                "✳  Skills & profile"
                            }
                            button { class: "text-button", disabled: signing_out(), onclick: handle_logout,
                                if signing_out() { "Signing out…" } else { "Sign out" }
                            }
                        },
                        Some(Ok(None)) => rsx! {
                            Link { to: Route::LoginPage {}, class: "nav-link", "Sign In" }
                            Link { to: Route::RegisterPage {}, class: "btn btn-primary btn-small", "Start writing ↗" }
                        },
                        Some(Err(_)) => rsx! { Link { to: Route::LoginPage {}, class: "nav-link", "Sign In" } },
                        None => rsx! { span { class: "muted", "Loading…" } },
                    }
                }
            }
            div { class: "topic-bar",
                span { class: "eyebrow", "EXPLORE" }
                button { class: if (search.0)().is_empty() { "topic active" } else { "topic" },
                    onclick: move |_| { search.0.set(String::new()); nav.push(Route::HomePage {}); }, "All stories" }
                for topic in ["Systems thinking", "Design", "Technology", "Digital gardens", "Culture"] {
                    button { class: if (search.0)() == topic { "topic active" } else { "topic" },
                        onclick: move |_| { search.0.set(topic.to_string()); nav.push(Route::HomePage {}); }, "{topic}" }
                }
            }
            div { class: "sidebar-note", span { "✳" } p { "Ideas get better" br {} "when you share them." } }
            if let Some(message) = logout_error() {
                div { class: "auth-error", role: "alert", "{message}" }
            }
        }
    }
}
