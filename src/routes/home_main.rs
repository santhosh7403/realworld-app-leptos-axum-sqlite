use crate::routes::profile_home::Profile;
use crate::routes::user_home::HomePage;
use leptos::prelude::*;
use leptos_meta::*;

/// Renders the home page of your application.
#[component]
pub fn HomeMain(username: crate::auth::UsernameSignal, user_profile: bool) -> impl IntoView {
    tracing::debug!("Starting HomePage component");

    view! {
        <div class="mx-auto sm:px-6 lg:px-8 bg-gray-200 dark:bg-gray-900 px-2 py-2">
            <Show
                when=move || !user_profile
                fallback=move || {
                    view! {
                        <Transition fallback=move || view! { <p>"Loading data..."</p> }>
                            <Profile username />
                        </Transition>
                    }
                }
            >
                <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                    <Title text="Home" />
                    <HomePage username />
                </Suspense>
            </Show>
        </div>
    }
}
