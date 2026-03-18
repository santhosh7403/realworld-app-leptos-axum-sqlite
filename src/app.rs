use crate::components::navitems::NavItems;
use crate::routes::{
    article_modal::*, editor_modal::*, home_main::*, login_modal::*, reset_password_modal::*,
    settings_modal::*, signup_modal::*, user_home::SearchAction,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Body, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use reactive_stores::Store;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
                <link
                    rel="stylesheet"
                    href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css"
                    integrity="sha512-DTOQO9RWCH3ppGqcWaEA1BIZOC6xxalwEsw9c2QQeAIftl+Vegovlnee1c9QX4TctnWMn13TZye+giMm8e2LwA=="
                    crossorigin="anonymous"
                    referrerpolicy="no-referrer"
                />

            </head>
            <body class="flex h-screen flex-col bg-gray-100">
                <App />
            </body>
        </html>
    }
}

#[derive(Clone, Debug, Store)]
pub struct GlobalState {
    back_url: String,
    is_profile: bool,
    home_url: String,
    search_results_window: bool,
    search_param: String,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            back_url: "/".to_string(),
            is_profile: false,
            home_url: "/".to_string(),
            search_results_window: false,
            search_param: String::new(),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_context(Store::new(GlobalState::default()));
    let per_page = RwSignal::new(Some(10u32));
    provide_context(per_page);
    let theme_mode = RwSignal::new("dark".to_string());
    provide_context(theme_mode);

    let username: crate::auth::UsernameSignal = RwSignal::new(None);
    let logout = ServerAction::<crate::auth::LogoutAction>::new();
    let login = ServerAction::<crate::auth::LoginAction>::new();
    let signup = ServerAction::<crate::auth::SignupAction>::new();

    let user = Resource::new(
        move || {
            (
                logout.version().get(),
                login.version().get(),
                signup.version().get(),
            )
        },
        move |_| {
            tracing::debug!("fetch user");
            crate::auth::current_user()
        },
    );

    let show_modal = RwSignal::new(false);
    provide_context(show_modal);

    Effect::new(move |_| {
        user.get().map(|x| {
            if let Ok(u) = x {
                username.set(Some(u.username()));
                per_page.set(Some(u.per_page_amount() as u32));
                theme_mode.set(u.theme_mode());
            } else {
                username.set(None);
            }
        });
    });

    let body_class = move || {
        let cls = if show_modal.get() {
            "bg-gray-800"
        } else if theme_mode.get() == "dark" {
            "bg-gray-900 text-white"
        } else {
            "bg-gray-100 text-black"
        };

        if theme_mode.get() == "dark" {
            format!("{} {}", cls, "dark")
        } else {
            cls.to_string()
        }
    };

    let footer_class = move || {
        if show_modal.get() {
            "hidden"
        } else {
            "bg-gray-200 text-gray-600 dark:bg-gray-900 dark:text-gray-400 text-center text-xs sticky bottom-0"
        }
    };

    let run_search = ServerAction::<SearchAction>::new();
    provide_context(run_search);

    view! {
        <Stylesheet id="leptos" href="/pkg/realworld-app-leptos-axum-sqlite.css" />
        <Body {..} class=body_class />

        // sets the document title
        <Title text="Welcome to Leptos" />

        <Router>
            <nav class=move || {
                format!(
                    "sticky top-0 z-10 shadow-md {}",
                    if show_modal.get() { "hidden" } else { "" },
                )
            }>
                <Transition fallback=|| {
                    view! { <p>"Loading Navigation bar"</p> }
                }>
                    {
                        view! { <NavItems login logout username /> }
                    }

                </Transition>
            </nav>
            <main>
                <Routes fallback=move || "Route Not found.">
                    <Route
                        path=path!("/")
                        view=move || view! { <HomeMain username user_profile=false /> }
                    />
                    <Route
                        path=path!("article/:slug")
                        view=move || view! { <Article username /> }
                    />
                    <Route path=path!("/login") view=move || view! { <LoginForm login /> } />
                    <Route
                        path=path!("/reset_password")
                        view=move || view! { <ResetPassword logout /> }
                    />
                    <Route path=path!("/signup") view=move || view! { <SignupForm signup /> } />
                    <Route path=path!("/settings") view=move || view! { <Settings logout /> } />
                    <Route path=path!("/editor") view=|| view! { <Editor /> } />
                    <Route path=path!("/editor/:slug") view=|| view! { <EditArticle /> } />
                    <Route
                        path=path!("/profile/:user")
                        view=move || {
                            view! { <HomeMain username user_profile=true /> }
                        }
                    />
                </Routes>
            </main>
            <footer class=footer_class>
                <a href="/">"MyApp"</a>
                <div class="bg-gray-200 text-gray-600 dark:bg-gray-900 dark:text-gray-400 text-center text-xs sticky bottom-0">
                    <p class="text-gray-600 dark:text-gray-400">"© 2026 My-Website"</p>
                </div>
            </footer>
        </Router>
    }
}

#[component]
fn EditArticle() -> impl IntoView {
    view! { <Editor /> }
}
