use dioxus::prelude::*;

use crate::enums::app::Route;
use crate::layouts::footer::Footer;
use crate::layouts::header::Header;

#[component]
pub fn AppLayout() -> Element {
    let route = use_route::<Route>();
    let section = match route {
        Route::HomePage {} => "The feed",
        Route::PublicationsPage {} | Route::PublicationPage { .. } => "Publications",
        Route::ProfilePage { .. } => "People",
        Route::WritePage {} | Route::EditPage { .. } => "Create",
        Route::DashboardPage {} => "Your stories",
        Route::LoginPage {} | Route::RegisterPage {} => "Welcome",
        _ => "Read & reflect",
    };
    rsx! {
        div {
            class: "app-shell",

            Header {}
            aside { class: "workspace-label", "aria-hidden": "true", span { "{section}" } }

            div {
                class: "app-body",


                main {
                    class: "main-content",

                    Outlet::<Route> {}
                }
            }

            Footer {}
        }
    }
}
