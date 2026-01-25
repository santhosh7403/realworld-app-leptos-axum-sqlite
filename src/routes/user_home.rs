use crate::app::{GlobalState, GlobalStateStoreFields};
use crate::components::article_view::ArticleView;
use crate::models::Pagination;
use leptos::{html::Input, prelude::*};
use leptos_meta::*;
use leptos_router::{
    components::A,
    hooks::{use_location, use_query},
    params::ParamsError,
};
use reactive_stores::Store;

use crate::components::{
    article_preview::ArticlePreviewList, items_per_page::ItemsPerPage,
    prev_next_buttons::PreviousNextButton,
};

#[tracing::instrument]
#[server(HomeAction, "/api", "GetJson")]
async fn home_articles(
    page: u32,
    amount: u32,
    tag: String,
    my_feed: bool,
) -> Result<Vec<crate::models::Article>, ServerFnError> {
    let page = i64::from(page);
    let amount = i64::from(amount);

    Ok(
        crate::models::Article::for_home_page(page, amount, tag, my_feed)
            .await
            .map_err(|x| {
                tracing::error!("problem while fetching home articles: {x:?}");
                ServerFnError::new("Problem while fetching home articles")
            })?,
    )
}

#[server(GetTagsAction, "/api", "GetJson")]
async fn get_tags() -> Result<Vec<String>, ServerFnError> {
    // sqlx::query!("SELECT DISTINCT tag FROM ArticleTags")
    sqlx::query!(
        "
            SELECT
                tag,
                tag_count,
                max_created_at
            FROM (
                SELECT
                    T1.tag,
                    COUNT(T1.article) AS tag_count,
                    MAX(T2.created_at) AS max_created_at
                FROM
                    ArticleTags AS T1
                JOIN
                    Articles AS T2
                ON
                    T1.article = T2.slug
                GROUP BY
                    T1.tag
            	ORDER BY  
            		tag_count DESC
            	LIMIT 10
            ) AS combined_tags

            UNION 

            SELECT
                tag,
                tag_count,
                max_created_at
            FROM (
                SELECT
                    T1.tag,
                    COUNT(T1.article) AS tag_count,
                    MAX(T2.created_at) AS max_created_at
                FROM
                    ArticleTags AS T1
                JOIN
                    Articles AS T2
                ON
                    T1.article = T2.slug
                GROUP BY
                    T1.tag
                ORDER BY
                    max_created_at DESC
                LIMIT 10
            ) AS combined_tags
        "
    )
    .map(|x| x.tag)
    .fetch_all(crate::database::get_db())
    .await
    .map(|tags| {
        tags.into_iter()
            .filter_map(|tag| tag)
            .collect::<Vec<String>>()
    })
    .map_err(|x| {
        tracing::error!("problem while fetching tags: {x:?}");
        ServerFnError::ServerError("Problem while fetching tags".into())
    })
}

#[server(SearchAction)]
pub async fn fetch_results(
    search: String,
    page: i64,
    amount: i64,
) -> Result<((i64, i64, i64), Vec<crate::models::MatchedArticles>), ServerFnError> {
    if search.is_empty() {
        Err(ServerFnError::new("Empty search string, hence ignore"))
    } else {
        let total = sqlx::query!(
            r#"SELECT
            COUNT(*) as "tot: i64" FROM Articles_fts AS AFTS JOIN  Articles AS A  ON A.oid = AFTS.rowid WHERE Articles_fts MATCH $1"#, search
        )
        .map(|x|x.tot)
        .fetch_one(crate::database::get_db())
        .await.map_err(|e| ServerFnError::new(format!("Some problem occured in sqlite query - {}", e.to_string())))
        ;

        Ok((
            (total.unwrap_or_default(), page, amount),
            crate::models::MatchedArticles::search_articles(search, page, amount)
                .await
                .map_err(|x| {
                    tracing::error!("problem while fetching search articles: {x:?}");
                    ServerFnError::new("Problem while fetching search articles")
                })?,
        ))
    }
}
/// Renders the home page of your application.
#[component]
pub fn HomePage(username: crate::auth::UsernameSignal) -> impl IntoView {
    let per_page: RwSignal<Option<u32>> =
        use_context().expect("per_page context should be available");
    tracing::debug!("Starting HomePage component");
    let pagination = use_query::<crate::models::Pagination>();

    let show_modal: RwSignal<bool> = use_context().expect("show_modal context should be available");
    show_modal.set(false); // Return from any Modal page should bring the home page front

    let articles = Resource::new(
        move || pagination.get().unwrap_or_default(),
        move |pagination| async move {
            tracing::debug!("making another request: {pagination:?}");
            home_articles(
                pagination.get_page(),
                pagination.get_amount(),
                pagination.get_tag().to_string(),
                pagination.get_my_feed(),
            )
            .await
        },
    );

    let global_state = expect_context::<Store<GlobalState>>();
    global_state.is_profile().set(false);

    let curr_location = format!(
        "{}{}",
        use_location().pathname.get_untracked(),
        if use_location().search.get_untracked().is_empty() {
            "".to_string()
        } else {
            format!("?{}", use_location().search.get_untracked())
        }
    );
    global_state.home_url().set(curr_location);
    global_state
        .back_url()
        .set(global_state.home_url().get_untracked());
    Effect::new(move || {
        global_state
            .home_url()
            .set(pagination.get().unwrap_or_default().to_string());
    });

    Effect::new(move || {
        pagination
            .get()
            .unwrap_or_default()
            .set_amount(per_page.get().unwrap());
    });

    let run_search = expect_context::<ServerAction<SearchAction>>();
    view! {
        <Title text="Home" />
        <div class="mx-auto lg:px-8 bg-gray-200 px-2 py-2 sm:px-0">
            <div class="">
                <div class="flex justify-between">
                    <div>
                        <YourFeedTab username pagination />
                        <GlobalFeedTab pagination />
                    </div>
                    <div>
                        <SearchArticle run_search />
                    </div>
                    <ItemsPerPage />
                </div>
                <SearchResults run_search username />
                <Show when=move || !global_state.search_results_window().get()>
                    <Show when=move || !pagination.get().unwrap_or_default().get_my_feed()>
                        <div class="flex gap-1 rounded bg-white mb-2">
                            <span class="font-bold m-1">Popular Tags:</span>
                            <TagList />
                        </div>
                    </Show>
                    <Show
                        when=move || {
                            articles
                                .with(|x| {
                                    x.as_ref()
                                        .map_or(0, |y| y.as_ref().map(Vec::len).unwrap_or_default())
                                }) != 0
                        }
                        fallback=move || {
                            view! {
                                <div>
                                    <p>
                                        {if pagination.get().unwrap_or_default().get_my_feed() {
                                            "You are not following any other user!"
                                        } else {
                                            "No articles to list"
                                        }}
                                    </p>
                                </div>
                            }
                        }
                    >
                        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                            <ArticlePreviewList username articles />
                        </Suspense>
                    </Show>
                </Show>
            </div>
            <Show when=move || !global_state.search_results_window().get()>
                <div class="flex gap-4">
                    <PreviousNextButton articles />
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn SearchResults(
    run_search: ServerAction<SearchAction>,
    username: crate::auth::UsernameSignal,
) -> impl IntoView {
    let global_state = expect_context::<Store<GlobalState>>();

    let articles_out = Resource::new(
        move || {
            (
                run_search.version().get(),
                global_state.search_results_window().get(),
            )
        },
        move |_| async move { run_search.value().get_untracked() },
    );

    let clear_search = move || {
        global_state.search_param().set(String::new());
        global_state.search_results_window().set(false);
        run_search.clear();
    };

    let open_article_cnt = RwSignal::new(0);
    let hide_all = RwSignal::new(false);

    let articles_view = move || {
        articles_out.with(move |x| {
            x.clone().map(move |res| {
                global_state.search_results_window().set(true);
                let (total_count, page, amount) = if let Some(Ok((t, _))) = res { t } else {global_state.search_results_window().set(false); (0, 0, 0) };
                // if total_count>0 {
                //     global_state.search_results_window().set(true)}else{global_state.search_results_window().set(false)}
                res.map(|res|{

                view! {
                    <Suspense fallback=move || {
                        view! { <p>"Loading..."</p> }
                    }>
                        <Show when=move || global_state.search_results_window().get()>
                            <div class="flex justify-between mb-1">
                                <div class="font-bold">"Search results = "{total_count}</div>
                                <div>
                                    <Show when=move || {
                                        total_count > 0
                                    }>
                                        "  Showing results "
                                        {match page {
                                            0 => 1,
                                            _ => page * amount + 1,
                                        }} " to "
                                        {match (page, total_count < amount) {
                                            (0, true) => total_count,
                                            (0, false) => amount,
                                            (_, _) => std::cmp::min((page + 1) * amount, total_count),
                                        }} " of "{total_count}
                                    </Show>
                                    <button
                                        class="text-blue-400 hover:underline hover:text-blue-500 transition duration-200 cursor-pointer px-8"
                                        on:click=move |_| clear_search()
                                    >
                                        "Clear Search"
                                    </button>
                                </div>
                                <ResultsViewPrevNextButton
                                    run_search
                                    page_data=(total_count, page, amount)
                                />
                            </div>
                        </Show>

                        <For
                            each=move || res.clone().unwrap_or_default().1.into_iter().enumerate()
                            key=|(i, _)| *i
                            children=move |(_, article)| {
                                view! {
                                    <SearchView
                                        article_res=article
                                        username
                                        open_article_cnt
                                        hide_all
                                    />
                                }
                            }
                        />
                        <ResultsViewPrevNextButton
                            run_search
                            page_data=(total_count, page, amount)
                        />
                    </Suspense>
                }
            })
            })
    })
    };
    view! {
        <Suspense fallback=move || view! { <p>"Loading Search Articles"</p> }>
            <ErrorBoundary fallback=|_| {
                view! { <p class="error-messages text-xs-center">"Something went wrong."</p> }
            }>{articles_view}</ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn ResultsViewPrevNextButton(
    run_search: ServerAction<SearchAction>,
    page_data: (i64, i64, i64),
) -> impl IntoView {
    let global_state = expect_context::<Store<GlobalState>>();

    let (total_count, page, amount) = page_data;

    view! {
        <div class="flex gap-2 justify-end">
            <Show when=move || { page > 0 }>
                <div>
                    <ActionForm action=run_search>
                        <input
                            type="hidden"
                            name="search"
                            value=move || { global_state.search_param().get() }
                        />
                        <input type="hidden" name="page" value=move || page - 1 />
                        <input type="hidden" name="amount" value=move || amount />
                        <button class="px-4 cursor-pointer rounded-full border hover:text-blue-500">
                            "Prev page"
                        </button>
                    </ActionForm>
                </div>

            </Show>

            <Show when=move || {
                match page {
                    0 => total_count > amount,
                    _ => total_count > ((page + 1) * amount),
                }
            }>
                <div>
                    <ActionForm action=run_search>
                        <input
                            type="hidden"
                            name="search"
                            value=move || { global_state.search_param().get() }
                        />
                        <input type="hidden" name="page" value=move || page + 1 />
                        <input type="hidden" name="amount" value=move || amount />
                        <button class="px-4 cursor-pointer rounded-full border hover:text-blue-500">
                            "Next page"
                        </button>
                    </ActionForm>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn SearchView(
    article_res: crate::models::MatchedArticles,
    username: crate::auth::UsernameSignal,
    open_article_cnt: RwSignal<i32>,
    hide_all: RwSignal<bool>,
) -> impl IntoView {
    let (show_article, set_show_article) = signal(false);
    let reset_show_article = move || {
        if hide_all.get() {
            set_show_article.set(false);
        }
    };

    view! {
        <div class="mb-2 p-4 bg-white rounded-lg shadow-md">
            <p>
                <span class="font-bold">"Title: "</span>
                <span inner_html=article_res.title></span>
            </p>
            <p>
                <span class="font-bold">"Description: "</span>
                <span inner_html=article_res.description></span>
            </p>
            <p>
                <span class="font-bold">"Body: "</span>
                <span inner_html=article_res.body></span>
            </p>
            <div class="flex justify-between">
                <div>
                    <A
                        href=format!("/article/{}", article_res.slug.clone())
                        target="_blank"
                        attr:class="text-blue-600 underline cursor-pointer"
                    >
                        "Open in a new tab/window"
                    </A>
                </div>
                <div>
                    <button
                        class="text-blue-600 underline cursor-pointer"
                        type="button"
                        on:click=move |_| {
                            set_show_article.set(!show_article.get());
                            match show_article.get() {
                                true => {
                                    open_article_cnt.update(|cnt| *cnt = *cnt + 1);
                                    hide_all.update(|h| *h = false)
                                }
                                false => open_article_cnt.update(|cnt| *cnt = *cnt - 1),
                            }
                        }
                    >
                        {move || {
                            reset_show_article();
                            format!(
                                "{}",
                                if show_article.get() { "Hide" } else { "Show Full Article" },
                            )
                        }}
                    </button>
                    <div>
                        <Show when=move || (open_article_cnt.get() > 1) && show_article.get()>
                            <button
                                class="text-blue-600 underline cursor-pointer"
                                type="button"
                                on:click=move |_| {
                                    hide_all.update(|h| *h = true);
                                    open_article_cnt.update(|cnt| *cnt = 0);
                                }
                            >
                                "Hide All"
                            </button>
                        </Show>
                    </div>
                </div>

            </div>
            <Show when=move || show_article.get() && !hide_all.get() fallback=|| ()>
                <ArticleView slug=article_res.slug.clone() username />
            </Show>
        </div>
    }
}

#[component]
fn SearchArticle(run_search: ServerAction<SearchAction>) -> impl IntoView {
    let search_string: NodeRef<Input> = NodeRef::new();
    let global_state = expect_context::<Store<GlobalState>>();

    let search_in = move |ev| global_state.search_param().set(event_target_value(&ev));
    let per_page: RwSignal<Option<u32>> =
        use_context().expect("per_page context should be available");

    view! {
        <ActionForm action=run_search>
            <div class="flex justify-end">
                <div class="flex justify-end">
                    <input
                        class="shadow appearance-none bg-white border rounded w-full py-1 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        type="text"
                        name="search"
                        minlength=2
                        placeholder="Search string"
                        required
                        node_ref=search_string
                        prop:value=move || global_state.search_param().get()
                        on:input=search_in
                    />
                    <input type="hidden" name="page" value=0 />
                    <input type="hidden" name="amount" value=move || per_page.get().unwrap() />
                    <button class="absolute pr-2 cursor-pointer hover:text-blue-500 transition duration-200 py-0.5">
                        <i class="fas fa-magnifying-glass"></i>
                    </button>
                </div>
            </div>
        </ActionForm>
    }
}

#[component]
fn YourFeedTab(
    username: RwSignal<Option<String>>,
    pagination: Memo<Result<Pagination, ParamsError>>,
) -> impl IntoView {
    let per_page: RwSignal<Option<u32>> =
        use_context().expect("per_page context should be available");
    let global_state = expect_context::<Store<GlobalState>>();

    view! {
        <button
            type="button"
            disabled=move || {
                username.with(Option::is_none) || global_state.search_results_window().get()
            }
            on:click=move |_| {
                let navigate = leptos_router::hooks::use_navigate();
                let your_feed_url = format!(
                    "{}",
                    if username.with(Option::is_some)
                        && !pagination
                            .with(|x| {
                                x.as_ref()
                                    .map(crate::models::Pagination::get_my_feed)
                                    .unwrap_or_default()
                            })
                    {
                        pagination
                            .get()
                            .unwrap_or_default()
                            .reset_page()
                            .set_my_feed(true)
                            .set_amount(per_page.get().unwrap())
                            .to_string()
                    } else {
                        String::from("/")
                    },
                );
                global_state.back_url().set(your_feed_url.clone());
                navigate(&your_feed_url, Default::default());
            }
            class=move || {
                format!(
                    "px-1 m-1 font-bold disabled:cursor-not-allowed {}",
                    if username.with(Option::is_none) {
                        "cursor-not-allowed bg-gray-200"
                    } else if pagination
                        .with(|x| {
                            x.as_ref()
                                .map(crate::models::Pagination::get_my_feed)
                                .unwrap_or_default()
                        })
                    {
                        "border-b-8 bg-gray-200"
                    } else {
                        "bg-gray-200 cursor-pointer"
                    },
                )
            }
        >
            "Your Feed"
        </button>
    }
}

#[component]
fn GlobalFeedTab(pagination: Memo<Result<Pagination, ParamsError>>) -> impl IntoView {
    let per_page: RwSignal<Option<u32>> =
        use_context().expect("per_page context should be available");
    let global_state = expect_context::<Store<GlobalState>>();

    view! {
        <button
            disabled=move || global_state.search_results_window().get()
            class=move || {
                format!(
                    "px-1 m-1 font-bold disabled:cursor-not-allowed {}",
                    if !pagination
                        .with(|x| {
                            x.as_ref()
                                .map(crate::models::Pagination::get_my_feed)
                                .unwrap_or_default()
                        })
                    {
                        "border-b-8 bg-gray-200"
                    } else {
                        "bg-gray-200 cursor-pointer"
                    },
                )
            }
            on:click=move |_| {
                let navigate = leptos_router::hooks::use_navigate();
                let global_feed_url = pagination
                    .get()
                    .unwrap_or_default()
                    .reset_page()
                    .set_my_feed(false)
                    .set_amount(per_page.get().unwrap())
                    .to_string();
                global_state.back_url().set(global_feed_url.clone());
                navigate(&global_feed_url, Default::default())
            }
        >
            "Global Feed"
        </button>
    }
}

#[component]
fn TagList() -> impl IntoView {
    let pagination = use_query::<crate::models::Pagination>();
    let tag_list = Resource::new(|| (), |_| async { get_tags().await });

    let tag_view = move || {
        let tag_elected = pagination.with(|x| {
            x.as_ref()
                .map(crate::models::Pagination::get_tag)
                .unwrap_or_default()
                .to_string()
        });
        tag_list.get().map(move |ts| {
            ts.map(move |tags| {
                view! {
                    <For
                        each=move || tags.clone().into_iter().enumerate()
                        key=|(i, _)| *i
                        children=move |(_, t): (usize, String)| {
                            let t2 = t.to_string();
                            let same = t2 == tag_elected;
                            view! {
                                <div class="gap-1">
                                    <a
                                        class=move || {
                                            format!(
                                                "rounded px-1 py-0.5 hover:bg-green-300 {}",
                                                if same { "bg-green-200" } else { "bg-gray-200" },
                                            )
                                        }
                                        href=move || {
                                            if same {
                                                pagination.get().unwrap_or_default().set_tag("").to_string()
                                            } else {
                                                pagination
                                                    .get()
                                                    .unwrap_or_default()
                                                    .set_tag(&t2)
                                                    .reset_page()
                                                    .to_string()
                                            }
                                        }
                                    >
                                        {t}
                                    </a>
                                </div>
                            }
                        }
                    />
                }
            })
        })
    };

    view! {
        <div class="flex gap-1 py-1 flex-wrap">
            <Suspense fallback=move || view! { <p>"Loading Tags"</p> }>
                <ErrorBoundary fallback=|_| {
                    view! { <p class="error-messages text-xs-center">"Something went wrong."</p> }
                }>{tag_view}</ErrorBoundary>
            </Suspense>
        </div>
    }
}
