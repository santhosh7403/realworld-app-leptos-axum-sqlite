use super::user_icons::AuthorUserIcon;
use crate::app::{GlobalState, GlobalStateStoreFields};
use leptos::prelude::*;
use leptos_router::{components::*, hooks::use_query_map};
use reactive_stores::Store;

use super::buttons::{ButtonFav, ButtonFavFavourited, ButtonFollow};
use crate::models::Article;

pub type ArticleSignal = RwSignal<crate::models::Article>;

#[component]
pub fn ArticlePreviewList(
    username: crate::auth::UsernameSignal,
    articles: Resource<Result<Vec<Article>, ServerFnError>>,
) -> impl IntoView {
    let articles_view = move || {
        articles.with(move |x| {
            x.clone().map(move |res| {
                view! {
                    <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                        <For
                            each=move || res.clone().unwrap_or_default().into_iter().enumerate()
                            key=|(i, _)| *i
                            children=move |(_, article): (usize, crate::models::Article)| {
                                let article = RwSignal::new(article);
                                view! { <ArticlePreview article=article username=username /> }
                            }
                        />
                    </Suspense>
                }
            })
        })
    };

    view! {
        <Suspense fallback=move || view! { <p>"Loading Articles"</p> }>
            <ErrorBoundary fallback=|_| {
                view! { <p class="error-messages text-xs-center">"Something went wrong."</p> }
            }>{articles_view}</ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn ArticlePreview(username: crate::auth::UsernameSignal, article: ArticleSignal) -> impl IntoView {
    let pagination = leptos_router::hooks::use_query::<crate::models::Pagination>();
    let per_page =
        use_context::<RwSignal<Option<u32>>>().expect("Should have the per_page signal from home");

    view! {
        <div class="mb-2 p-4 bg-white dark:bg-gray-800 dark:text-gray-100 rounded-lg shadow-md">
            <div class="flex items-center gap-4 mb-4">
                <ArticleMeta username=username article=article is_preview=true />
            </div>
            <A href=move || format!("/article/{}", article.with(|x| x.slug.clone()))>
                <h2 class="text-2xl font-bold mb-2 text-gray-800 dark:text-gray-300">
                    {move || article.with(|x| x.title.to_string())}
                </h2>
            </A>
            <A href=move || format!("/article/{}", article.with(|x| x.slug.clone()))>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    {move || article.with(|x| x.description.to_string())}
                </p>
            </A>

            <div class="flex justify-between items-end">
                <A href=move || format!("/article/{}", article.with(|x| x.slug.clone()))>
                    <span class="hover:text-blue-600 hover:underline cursor-pointer">
                        "Read more..."
                    </span>
                </A>
                <Show when=move || {
                    article.with(|x| !x.tag_list.is_empty() && x.tag_list.first().unwrap() != "")
                }>
                    // fallback=|| view! { <span>"No tags"</span> }
                    <div class="flex flex-wrap gap-1">
                        <i class="fa-solid fa-hashtag py-1"></i>
                        <For
                            each=move || {
                                article.with(|x| x.tag_list.clone().into_iter().enumerate())
                            }
                            key=|(i, _)| *i
                            children=move |(_, tag): (usize, String)| {
                                let tag_now = tag.clone();
                                view! {
                                    <span class="bg-gray-200 dark:bg-gray-700 dark:text-gray-200 text-gray-700 dark:text-gray-300 px-2 py-1 rounded text-xs flex items-center gap-1">
                                        <A href=move || {
                                            pagination()
                                                .unwrap_or_default()
                                                .set_tag(&tag_now)
                                                .set_amount(per_page.get().unwrap())
                                                .to_string()
                                        }>{tag}</A>
                                    </span>
                                }
                            }
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn ArticleMeta(
    username: crate::auth::UsernameSignal,
    article: ArticleSignal,
    is_preview: bool,
) -> impl IntoView {
    let editor_ref = move || format!("/editor/{}", article.with(|x| x.slug.to_string()));
    let article_owner = username.get_untracked().unwrap_or_default()
        == article.with_untracked(|x| x.author.username.to_string());

    let (author, set_author) = signal(String::new());
    let author_user = move || {
        set_author(article.with(|x| x.author.username.to_string()));
    };
    let global_state = expect_context::<Store<GlobalState>>();
    let back_url = move || global_state.back_url().get().to_string();

    let delete_a = ServerAction::<DeleteArticleAction>::new();

    let query = use_query_map();
    let favourite = move || query.with(|x| x.get("favourites").map(|_| true));

    view! {
        <div class="article-meta">
            <div class="flex items-center gap-4 text-gray-700 dark:text-gray-300">
                <Show when=move || is_preview>
                    <AuthorUserIcon article_signal=article />
                </Show>
                <div class="flex items-center gap-1">
                    <span class="">
                        <i class="fa-solid fa-calendar w-4 h-4"></i>
                        {move || article.with_untracked(|x| x.created_at.to_string())}
                    </span>
                </div>
                <div class="flex items-center gap-1">
                    <i class=move || {
                        format!(
                            "{}",
                            if article.with(|x| x.comments_count) > 0 {
                                "fas fa-comments w-4 h-4 text-yellow-500"
                            } else {
                                "far fa-comments w-4 h-4"
                            },
                        )
                    }></i>
                    <span class="px-1">
                        " Comments: " {move || article.with(|x| x.comments_count)}
                    </span>
                </div>
                <Show
                    when=move || is_preview
                    fallback=move || {
                        view! {
                            <Show
                                when=move || article_owner
                                fallback=move || {
                                    view! {
                                        <Show
                                            when=move || {
                                                author_user();
                                                username.with(Option::is_some)
                                            }
                                            fallback=move || {
                                                view! { <ButtonFav username article /> }
                                            }
                                        >
                                            <ButtonFav username article />
                                            <ButtonFollow logged_user=username author />
                                        </Show>
                                    }
                                }
                            >
                                <A href=editor_ref>
                                    <i class="fa-solid fa-pen-to-square w-4 h-4"></i>
                                    " Edit Article"
                                </A>
                                <ActionForm action=delete_a>
                                    <input
                                        type="hidden"
                                        name="slug"
                                        value=move || article.with(|x| x.slug.to_string())
                                    />
                                    <input type="hidden" name="back_url" value=back_url />
                                    <button
                                        type="submit"
                                        class="text-red-400 hover:rounded hover:border hover:bg-red-100"
                                    >
                                        <i class="fa-solid fa-trash-can w-4 h-4"></i>
                                        " Delete Article"
                                    </button>
                                </ActionForm>
                            </Show>
                        }
                    }
                >
                    <Show
                        when=move || favourite().unwrap_or_default()
                        fallback=move || {
                            view! { <ButtonFav username=username article=article /> }
                        }
                    >
                        <ButtonFavFavourited article />
                    </Show>
                </Show>
            </div>
        </div>
    }
}

#[server(DeleteArticleAction, "/api")]
#[tracing::instrument]
pub async fn delete_article(slug: String, back_url: String) -> Result<(), ServerFnError> {
    let Some(logged_user) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Article::delete(slug, logged_user)
        .await
        .map(move |_| {
            // leptos_axum::redirect("/");
            leptos_axum::redirect(&back_url);
        })
        .map_err(|x| {
            let err = format!("Error while deleting an article: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not delete the article, try again later".into())
        })
}
