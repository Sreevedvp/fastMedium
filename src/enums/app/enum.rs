use dioxus::prelude::*;

use crate::layouts::layout::AppLayout;
use crate::pages::dashboard::DashboardPage;
use crate::pages::edit::EditPage;
use crate::pages::home::HomePage;
use crate::pages::login::LoginPage;
use crate::pages::post::PostPage;
use crate::pages::profile::ProfilePage;
use crate::pages::publications::{PublicationPage, PublicationsPage};
use crate::pages::reference::ReferencePage;
use crate::pages::register::RegisterPage;
use crate::pages::write::WritePage;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
        #[route("/")]
        HomePage {},

        #[route("/publications")]
        PublicationsPage {},

        #[route("/publication/:slug")]
        PublicationPage { slug: String },

        #[route("/preview")]
        ReferencePage {},

        #[route("/login")]
        LoginPage {},

        #[route("/register")]
        RegisterPage {},

        #[route("/dashboard")]
        DashboardPage {},

        #[route("/write")]
        WritePage {},

        #[route("/edit/:id")]
        EditPage { id: String },

        #[route("/post/:slug")]
        PostPage { slug: String },

        #[route("/profile/:username")]
        ProfilePage { username: String },
}
