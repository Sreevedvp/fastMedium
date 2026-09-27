use crate::{
    app::AuthVersion,
    client::{auth::get_current_user, posts::get_user_posts, publications as api},
    enums::app::Route,
    models::{post::PostSummary, publication::Publication},
};
use dioxus::prelude::*;

#[component]
pub fn PublicationsPage() -> Element {
    let version = use_context::<AuthVersion>();
    let me = use_resource(move || {
        let _ = (version.0)();
        get_current_user()
    });
    let publications = use_resource(api::list);
    let mut creating = use_signal(|| false);
    let authenticated = matches!(&*me.read(), Some(Ok(Some(_))));
    rsx! {
        section { class: "collections-page",
            p { class: "eyebrow", "MAKE A LITTLE ROOM" }
            div { class: "page-heading", h1 { "Big ideas. " em { "Shared spaces." } }
                if authenticated { button { class: "btn btn-primary", onclick: move |_| creating.toggle(), if creating() { "Close form" } else { "＋ Create publication" } } }
                else { Link { to: Route::LoginPage {}, class: "btn btn-primary", "Sign in to create" } }
            }
            p { class: "page-intro", "Build a home for a subject you love. Bring your stories together in a publication that feels like you." }
            if creating() && authenticated { CreatePublication {} }
            match &*publications.read() {
                Some(Ok(items)) => rsx! {
                    div { class: "collection-grid",
                        for item in items {
                            Link { key: "{item.id}", to: Route::PublicationPage { slug: item.slug.clone() }, class: "collection-card",
                                span { class: "collection-icon", "▤" }
                                h2 { "{item.name}" }
                                p { "{item.description}" }
                                div { class: "collection-meta", span { "@{item.owner_username}" } span { "{item.story_count} published · ↗" } }
                            }
                        }
                    }
                    if items.is_empty() { div { class: "empty-feed", h2 { "A fresh page, waiting for you." } p { "Create the first publication and give your ideas a place to belong." } } }
                },
                Some(Err(e)) => rsx! { p { class: "error-state", "{e}" } },
                None => rsx! { p { class: "loading-state", "Loading publications…" } },
            }
        }
    }
}

#[component]
fn CreatePublication() -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let nav = use_navigator();
    rsx! {
        form { class: "publication-form", onsubmit: move |e| {
            e.prevent_default(); if busy() { return; }
            busy.set(true); error.set(None);
            spawn(async move { match api::create(name(), description()).await { Ok(p) => { nav.push(Route::PublicationPage { slug: p.slug }); }, Err(e) => error.set(Some(e)) }; busy.set(false); });
        },
            h2 { "Your new publication" }
            label { r#for: "publication-name", "Name" }
            input { id: "publication-name", placeholder: "The curious corner", required: true, minlength: 3, maxlength: 80, value: "{name}", oninput: move |e| name.set(e.value()) }
            label { r#for: "publication-description", "What is it about?" }
            textarea { id: "publication-description", placeholder: "A small space for big conversations about…", maxlength: 500, rows: 3, value: "{description}", oninput: move |e| description.set(e.value()) }
            if let Some(e) = error() { p { class: "auth-error", role: "alert", "{e}" } }
            button { class: "btn btn-primary", r#type: "submit", disabled: busy(), if busy() { "Creating…" } else { "Create publication ↗" } }
        }
    }
}

#[component]
pub fn PublicationPage(slug: String) -> Element {
    let version = use_context::<AuthVersion>();
    let mut data = use_resource(use_reactive((&slug,), move |(slug,)| async move {
        let _ = (version.0)();
        api::detail(slug).await
    }));
    rsx! {
        section { class: "collection-detail",
            Link { to: Route::PublicationsPage {}, class: "back-link", "← All publications" }
            match &*data.read() {
                Some(Ok(detail)) => rsx! {
                    div { class: "collection-hero",
                        span { class: "collection-icon", "▤" }
                        p { class: "eyebrow", "AN INDEPENDENT PUBLICATION" }
                        h1 { "{detail.publication.name}" }
                        p { class: "page-intro", "{detail.publication.description}" }
                        Link { to: Route::ProfilePage { username: detail.publication.owner_username.clone() }, "Curated by @{detail.publication.owner_username}" }
                    }
                    if detail.can_manage {
                        PublicationSettings { publication: detail.publication.clone(), on_saved: move |_| data.restart() }
                        StoryPicker { publication_id: detail.publication.id.clone(), attached: detail.posts.iter().map(|p| p.id.clone()).collect::<Vec<_>>(), on_saved: move |_| data.restart() }
                    }
                    div { class: "section-heading", h2 { "The stories" } span { "{detail.publication.story_count} published" } }
                    for post in &detail.posts {
                        CollectionStory { key: "{post.id}", post: post.clone(), publication_id: detail.publication.id.clone(), can_manage: detail.can_manage, on_saved: move |_| data.restart() }
                    }
                    if detail.posts.is_empty() { div { class: "empty-feed", p { "No stories here yet. Good things take a little time." } } }
                },
                Some(Err(e)) => rsx! { p { class: "error-state", role: "alert", "{e}" } },
                None => rsx! { p { class: "loading-state", "Opening publication…" } },
            }
        }
    }
}

#[component]
fn PublicationSettings(publication: Publication, on_saved: EventHandler<()>) -> Element {
    let mut name = use_signal(|| publication.name.clone());
    let mut description = use_signal(|| publication.description.clone());
    let mut editing = use_signal(|| false);
    let mut confirming = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let id = publication.id.clone();
    let nav = use_navigator();
    rsx! {
        div { class: "collection-controls",
            button { class: "btn btn-outline", onclick: move |_| editing.toggle(), if editing() { "Close settings" } else { "Edit publication" } }
            button { class: "text-button", onclick: move |_| confirming.toggle(), "Delete publication" }
        }
        if editing() {
            form { class: "publication-form", onsubmit: move |e| {
                e.prevent_default(); if busy() { return; } let id = id.clone(); busy.set(true); error.set(None);
                spawn(async move { match api::update(id, name(), description()).await { Ok(_) => { editing.set(false); on_saved.call(()); }, Err(e) => error.set(Some(e)) }; busy.set(false); });
            },
                label { r#for: "edit-publication-name", "Publication name" }
                input { id: "edit-publication-name", required: true, minlength: 3, maxlength: 80, value: "{name}", oninput: move |e| name.set(e.value()) }
                label { r#for: "edit-publication-description", "Description" }
                textarea { id: "edit-publication-description", maxlength: 500, rows: 3, value: "{description}", oninput: move |e| description.set(e.value()) }
                button { class: "btn btn-primary", disabled: busy(), "Save publication" }
            }
        }
        if confirming() {
            div { class: "confirmation", role: "alert",
                p { "Delete this publication? Your stories will stay in My stories." }
                button { class: "btn btn-danger", disabled: busy(), onclick: move |_| {
                    let id = publication.id.clone(); busy.set(true); error.set(None);
                    spawn(async move { match api::delete(id).await { Ok(_) => { nav.push(Route::PublicationsPage {}); }, Err(e) => error.set(Some(e)) }; busy.set(false); });
                }, "Yes, delete publication" }
                button { class: "btn btn-outline", onclick: move |_| confirming.set(false), "Cancel" }
            }
        }
        if let Some(e) = error() { p { class: "auth-error", role: "alert", "{e}" } }
    }
}

#[component]
fn StoryPicker(
    publication_id: String,
    attached: Vec<String>,
    on_saved: EventHandler<()>,
) -> Element {
    let mine = use_resource(get_user_posts);
    let mut selected = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        section { class: "story-picker",
            h3 { "Add a story to this publication" }
            p { class: "field-hint", "Choose one of your stories. Drafts stay private. A story belongs to one publication at a time; adding it here moves it from its previous publication." }
            match &*mine.read() {
                Some(Ok(items)) => rsx! {
                    form { class: "inline-form", onsubmit: move |e| {
                        e.prevent_default(); if busy() || selected().is_empty() { return; } let id = publication_id.clone();
                        busy.set(true); error.set(None);
                        spawn(async move { match api::attach(id, selected()).await { Ok(_) => { selected.set(String::new()); on_saved.call(()); }, Err(e) => error.set(Some(e)) }; busy.set(false); });
                    },
                        select { "aria-label": "Story to add", value: "{selected}", onchange: move |e| selected.set(e.value()),
                            option { value: "", "Choose a story…" }
                            for post in items.iter().filter(|p| !attached.contains(&p.id)) {
                                option { value: "{post.id}", "{post.title}" if post.published_at.is_none() { " (draft)" } }
                            }
                        }
                        button { class: "btn btn-primary", disabled: busy() || selected().is_empty(), "Add story" }
                    }
                    if items.is_empty() { Link { to: Route::WritePage {}, "Write your first story →" } }
                },
                Some(Err(e)) => rsx! { p { class: "auth-error", "{e}" } },
                None => rsx! { p { "Loading your stories…" } },
            }
            if let Some(e) = error() { p { class: "auth-error", role: "alert", "{e}" } }
        }
    }
}

#[component]
fn CollectionStory(
    post: PostSummary,
    publication_id: String,
    can_manage: bool,
    on_saved: EventHandler<()>,
) -> Element {
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        article { class: "collection-story",
            Link { to: if post.published_at.is_some() { Route::PostPage { slug: post.slug.clone() } } else { Route::EditPage { id: post.id.clone() } },
                p { class: "eyebrow", if post.published_at.is_some() { "PUBLISHED STORY" } else { "PRIVATE DRAFT" } }
                h3 { "{post.title}" } p { "{post.excerpt}" }
            }
            if can_manage {
                button { class: "text-button", disabled: busy(), onclick: move |_| {
                    let id = publication_id.clone(); let pid = post.id.clone(); busy.set(true);
                    spawn(async move { match api::detach(id, pid).await { Ok(_) => on_saved.call(()), Err(e) => error.set(Some(e)) }; busy.set(false); });
                }, "Remove from publication" }
            }
            if let Some(e) = error() { p { class: "auth-error", "{e}" } }
        }
    }
}
