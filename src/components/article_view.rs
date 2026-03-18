use super::buttons::ButtonFav;
use crate::components::article_preview::ArticleSignal;
use crate::components::user_icons::AuthorUserIcon;
use crate::models::User;
use crate::routes::article_modal::{get_article, ArticleResult, CommentSection};
use leptos::prelude::*;
use leptos_meta::Title;

#[tracing::instrument]
#[component]
pub fn ArticleView(slug: String, username: crate::auth::UsernameSignal) -> impl IntoView {
    let article = LocalResource::new(move || {
        let slug = slug.clone();
        async { get_article(slug).await }
    });

    view! {
        <Title text="Search Results" />

        <Suspense fallback=move || view! { <p>"Loading Article"</p> }>
            <ErrorBoundary fallback=|_| {
                view! {
                    <p class="error-messages text-xs-center">
                        "Something went wrong, please try again later."
                    </p>
                }
            }>
                {move || {
                    article
                        .get()
                        .map(move |x| {
                            x.map(move |article_result| {
                                view! { <ArticleViewPage username result=article_result /> }
                            })
                        })
                }}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn ArticleViewPage(username: crate::auth::UsernameSignal, result: ArticleResult) -> impl IntoView {
    let article_signal = RwSignal::new(result.article.clone());
    let user_signal = RwSignal::new(result.logged_user);

    view! { <ArticleViewPageModal username article_signal user_signal /> }
}

#[component]
pub fn ArticleViewPageModal(
    username: crate::auth::UsernameSignal,
    article_signal: ArticleSignal,
    user_signal: RwSignal<Option<User>>,
) -> impl IntoView {
    view! {
        <div class="bg-opacity-60 inset-0 flex items-center justify-center">
            <div class="block w-4/5 rounded-lg bg-white dark:bg-gray-800 dark:text-gray-100 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)]">
                <div class="mb-5 px-1 py-1">
                    <div class="mb-5">
                        <ArticleMetaForView username article=article_signal />
                    </div>
                    <div class="flex justify-between mb-5">
                        <div>
                            <div class="mb-2">
                                <h1 class="text-xl leading-tight font-medium text-neutral-800 dark:text-neutral-100">
                                    {article_signal.get_untracked().title}
                                </h1>
                            </div>
                        </div>
                    </div>
                    <div class="mb-5">
                        <p>{article_signal.get_untracked().body}</p>
                    </div>
                </div>
                <div class="mb-5 px-1 py-1">
                    <CommentSection username article=article_signal user_signal />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ArticleMetaForView(
    username: crate::auth::UsernameSignal,
    article: ArticleSignal,
) -> impl IntoView {
    view! {
        <div class="article-meta">
            <div class="flex items-center gap-4 text-gray-700 dark:text-gray-300">
                <AuthorUserIcon article_signal=article />
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
                <ButtonFav username=username article=article />
            </div>
        </div>
    }
}
