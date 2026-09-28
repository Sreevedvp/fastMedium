mod api_types;
mod app;
mod client;
mod components;
mod enums;
mod layouts;
mod models;
mod pages;
mod ui;

fn main() {
    dioxus::launch(app::App);
}
