use crate::app::{GlobalState, GlobalStateStoreFields};
use crate::models::Article;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query, use_query_map};
use reactive_stores::Store;

#[component]
pub fn PreviousNextButton(
    articles: Resource<Result<Vec<Article>, ServerFnError>>,
) -> impl IntoView {
    let params = use_params_map();
    let route_user = move || params.with(|x| x.get("user").unwrap_or_default());
    let query = use_query_map();
    let favourite = move || query.with(|x| x.get("favourites").map(|_| true));

    let global_state = expect_context::<Store<GlobalState>>();
    let pagination = use_query::<crate::models::Pagination>();

    view! {
        <Show
            when=move || {
                pagination
                    .with(|x| {
                        x.as_ref().map(crate::models::Pagination::get_page).unwrap_or_default()
                    }) > 0
            }
            fallback=|| ()
        >
            <button
                type="button"
                class="px-4 cursor-pointer hover:text-blue-500 border dark:border-gray-600 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200"
                // class="px-4 cursor-pointer hover:text-blue-500 border rounded-full bg-gray-100"
                on:click=move |_| {
                    let prev_page = format!(
                        "{}{}{}",
                        if global_state.is_profile().get() {
                            format!("/profile/{}", route_user())
                        } else {
                            "".to_string()
                        },
                        pagination.get().unwrap_or_default().previous_page().to_string(),
                        if favourite().unwrap_or_default() { "&favourites=true" } else { "" },
                    );
                    let navigate = leptos_router::hooks::use_navigate();
                    global_state.back_url().set(prev_page.clone());
                    navigate(&prev_page, Default::default())
                }
            >
                "<< Previous page      "
            </button>
        </Show>
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <Show
                when=move || {
                    let n_articles = articles
                        .with(|x| {
                            x.as_ref().map_or(0, |y| y.as_ref().map(Vec::len).unwrap_or_default())
                        });
                    n_articles > 0
                        && n_articles
                            >= pagination
                                .with(|x| {
                                    x.as_ref()
                                        .map(crate::models::Pagination::get_amount)
                                        .unwrap_or_default()
                                }) as usize
                }
                fallback=|| ()
            >
                <button
                    type="button"
                    class="px-4 cursor-pointer hover:text-blue-500 border dark:border-gray-600 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200"
                    on:click=move |_| {
                        let next_page = format!(
                            "{}{}{}",
                            if global_state.is_profile().get() {
                                format!("/profile/{}", route_user())
                            } else {
                                "".to_string()
                            },
                            pagination.get().unwrap_or_default().next_page().to_string(),
                            if favourite().unwrap_or_default() { "&favourites=true" } else { "" },
                        );
                        let navigate = leptos_router::hooks::use_navigate();
                        global_state.back_url().set(next_page.clone());
                        navigate(&next_page, Default::default())
                    }
                >
                    "Next page >>"
                </button>
            </Show>
        </Suspense>
    }
}
