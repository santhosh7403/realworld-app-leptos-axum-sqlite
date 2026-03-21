use crate::app::{GlobalState, GlobalStateStoreFields};
use crate::components::{
    article_preview::ArticlePreviewList, items_per_page::ItemsPerPage,
    prev_next_buttons::PreviousNextButton,
};
use crate::models::Pagination;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    hooks::{use_params_map, use_query, use_query_map},
    params::ParamsError,
};
use reactive_stores::Store;

#[server(UserArticlesAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn profile_articles(
    username: String,
    favourites: Option<bool>,
    page: u32,
    amount: u32,
) -> Result<Vec<crate::models::Article>, ServerFnError> {
    let page = i64::from(page);
    let amount = i64::from(amount);

    crate::models::Article::for_user_profile_home(
        username,
        favourites.unwrap_or_default(),
        page,
        amount,
    )
    .await
    .map_err(|x| {
        let err = format!("Error while getting user_profile articles: {x:?}");
        tracing::error!("{err}");
        ServerFnError::ServerError("Could not retrieve articles, try again later".into())
    })
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UserProfileModel {
    user: crate::models::User,
    following: Option<bool>,
}

#[server(UserProfileAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn user_profile(username: String) -> Result<UserProfileModel, ServerFnError> {
    let user = crate::models::User::get(username.clone())
        .await
        .map_err(|x| {
            let err = format!("Error while getting user in user_profile: {x:?}");
            tracing::error!("{err}");
            ServerFnError::new("Could not retrieve articles, try again later")
        })?;
    let mut following = None;
    if let Some(logged_user) = crate::auth::get_username() {
        if sqlx::query_scalar!(
            "
            Select count(*) from Follows where follower=$2 and influencer=$1
            ",
            username,
            logged_user
        )
        .fetch_one(crate::database::get_db())
        .await?
            == 1
        {
            following = Some(true);
        }
    }
    Ok(UserProfileModel { user, following })
}

// #[allow(clippy::redundant_closure)]
#[tracing::instrument]
#[component]
pub fn Profile(username: crate::auth::UsernameSignal) -> impl IntoView {
    let params = use_params_map();

    // let route_user = move || params.with(|x| x.get("user").unwrap_or_default());
    let route_user = Memo::new(move |_| params.with(|x| x.get("user").unwrap_or_default()));

    let global_state = expect_context::<Store<GlobalState>>();
    global_state.is_profile().set(true);

    let show_modal: RwSignal<bool> = use_context().expect("show_modal context should be available");
    show_modal.set(false); // profile page is wired to home, so lets show the menu, if its called from a modal like article detail page

    let on_back_event = move || {
        let navigate = leptos_router::hooks::use_navigate();
        navigate(
            &global_state.home_url().get().to_string(),
            Default::default(),
        );
    };

    view! {
        <Title text=move || format!("{}'s profile", route_user.get()) />
        <div>
            <ProfileHome on_back_event username route_user />
        </div>
    }
}

#[component]
pub fn ProfileHome<C>(
    on_back_event: C,
    username: crate::auth::UsernameSignal,
    route_user: Memo<String>,
) -> impl IntoView
where
    C: Fn() + 'static + Copy + Send,
{
    let pagination = use_query::<crate::models::Pagination>();
    let per_page: RwSignal<Option<u32>> =
        use_context().expect("per_page context should be available");

    let query = use_query_map();
    let favourite = move || query.with(|x| x.get("favourites").map(|_| true));

    let articles = Resource::new(
        move || {
            (
                favourite(),
                route_user.get(),
                pagination.get().unwrap_or_default().get_page(),
                per_page.get().unwrap(),
            )
        },
        move |(fav, user, page, amount)| async move { profile_articles(user, fav, page, amount).await },
    );

    let global_state = expect_context::<Store<GlobalState>>();
    global_state
        .back_url()
        .set(format!("/profile/{}", route_user.get_untracked()));
    view! {
        <div class="px-1">
            <div class="mb-5">
                <div class="flex justify-between px-2 bg-gray-200 dark:bg-gray-700">
                    <div class="flex">
                        <UserArticlesTab favourite route_user pagination />
                        <FavouritedArticlesTab favourite route_user pagination />
                    </div>
                    <ItemsPerPage username/>
                </div>
                <UserInfo on_back_event />
                <ArticlePreviewList username articles />
                <div class="flex justify-between">
                    <div class="flex gap-4">
                        <PreviousNextButton articles />
                    </div>
                    <div class="flex justify-end px-7">
                        <BackToHome on_back_event />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn UserArticlesTab<A>(
    favourite: A,
    route_user: Memo<String>,
    pagination: Memo<Result<Pagination, ParamsError>>,
) -> impl IntoView
where
    A: Fn() -> Option<bool> + 'static + Send,
{
    let per_page: RwSignal<Option<u32>> =
        use_context().expect("per_page context should be available");
    let global_state = expect_context::<Store<GlobalState>>();

    view! {
        <div class="mb-5 px-2">
            <button
                type="button"
                class=move || {
                    format!(
                        "font-bold {}",
                        if !favourite().unwrap_or_default() {
                            "border-b-8"
                        } else {
                            "cursor-pointer"
                        },
                    )
                }
                on:click=move |_| {
                    let navigate = leptos_router::hooks::use_navigate();
                    let profile_url = format!(
                        "/profile/{}{}",
                        route_user.get_untracked(),
                        pagination
                            .get()
                            .unwrap_or_default()
                            .reset_page()
                            .set_amount(per_page.get().unwrap())
                            .to_string(),
                    );
                    global_state.back_url().set(profile_url.clone());
                    navigate(&profile_url, Default::default());
                }
            >
                {move || route_user.get()}
                "'s Articles"
            </button>
        </div>
    }
}

#[component]
fn FavouritedArticlesTab<A>(
    favourite: A,
    route_user: Memo<String>,
    pagination: Memo<Result<Pagination, ParamsError>>,
) -> impl IntoView
where
    A: Fn() -> Option<bool> + 'static + Send,
{
    let per_page: RwSignal<Option<u32>> =
        use_context().expect("per_page context should be available");
    let global_state = expect_context::<Store<GlobalState>>();

    view! {
        <div class="mb-5 px-2">
            <button
                type="button"
                class=move || {
                    format!(
                        "font-bold {}",
                        if favourite().unwrap_or_default() {
                            "border-b-8"
                        } else {
                            "cursor-pointer"
                        },
                    )
                }
                on:click=move |_| {
                    let navigate = leptos_router::hooks::use_navigate();
                    let favourited_url = format!(
                        "/profile/{}{}{}",
                        route_user.get_untracked(),
                        pagination
                            .get()
                            .unwrap_or_default()
                            .reset_page()
                            .set_amount(per_page.get().unwrap())
                            .to_string(),
                        "&favourites=true",
                    );
                    global_state.back_url().set(favourited_url.clone());
                    navigate(&favourited_url, Default::default())
                }
            >
                "Favourited Articles"
            </button>
        </div>
    }
}

#[component]
pub fn UserInfo<C>(on_back_event: C) -> impl IntoView
where
    C: Fn() + 'static + Copy + Send,
{
    let params = use_params_map();
    let resource = Resource::new(
        move || params.with(|x| x.get("user").clone().unwrap_or_default()),
        move |user| async move { user_profile(user).await },
    );

    view! {
        <div class="gap-4 shadow-md rounded-lg mb-3 px-2 py-2 bg-white dark:bg-gray-800 dark:text-gray-100">
            <Transition fallback=move || view! { <p>"Loading user profile"</p> }>
                <ErrorBoundary fallback=|_| {
                    view! {
                        <p>
                            "There was a problem while fetching the user profile, try again later"
                        </p>
                    }
                }>
                    {move || {
                        resource
                            .get()
                            .map(move |x| {
                                x.map(move |u| {
                                    let image = u.user.image();
                                    let username = u.user.username();
                                    let bio = u.user.bio();
                                    let email = format!(
                                        "{}",
                                        if u.user.email().is_empty() {
                                            " - ".to_string()
                                        } else {
                                            u.user.email()
                                        },
                                    );

                                    view! {
                                        <div>
                                            <div class="mb-5 px-5 flex justify-between">
                                                <h2 class="font-bold text-xl underline">
                                                    "Profile data of User - "{username.clone()}
                                                </h2>
                                                <BackToHome on_back_event />
                                            </div>
                                            <div class="flex">
                                                <div class="mb-4">
                                                    <img src=image class="w-10 h-10 rounded-full" />
                                                </div>
                                                <div class="px-4">
                                                    <h4>{username}</h4>
                                                </div>
                                            </div>
                                            <p class="">
                                                "Bio: "{bio.unwrap_or("No bio available".into())}
                                            </p>
                                            <div class="mb-5">"Email: "{email}</div>
                                        </div>
                                    }
                                })
                            })
                    }}
                </ErrorBoundary>
            </Transition>
        </div>
    }
}

#[component]
fn BackToHome<C>(on_back_event: C) -> impl IntoView
where
    C: Fn() + 'static + Clone,
{
    view! {
        <h4 class="text-blue-500 underline cursor-pointer" on:click=move |_| on_back_event()>
            Back to Home
        </h4>
    }
}
