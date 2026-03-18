use crate::app::{GlobalState, GlobalStateStoreFields};
use crate::components::article_preview::{ArticleMeta, ArticleSignal};
use crate::components::user_icons::{AuthorUserIcon, CommentUserIcon, CurrentUserIcon};
use crate::models::{Comment, User};
use leptos::html::Textarea;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use reactive_stores::Store;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct ArticleResult {
    pub article: crate::models::Article,
    pub logged_user: Option<crate::models::User>,
}

#[derive(Clone)]
pub struct FollowUser(pub bool);

#[server(GetArticleAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn get_article(slug: String) -> Result<ArticleResult, ServerFnError> {
    Ok(ArticleResult {
        article: crate::models::Article::for_article(slug)
            .await
            .map_err(|x| {
                let err = format!("Error while getting user_profile articles: {x:?}");
                tracing::error!("{err}");
                ServerFnError::new("Could not retrieve articles, try again later")
            })?,
        logged_user: crate::auth::current_user().await.ok(),
    })
}

#[tracing::instrument]
#[component]
pub fn Article(username: crate::auth::UsernameSignal) -> impl IntoView {
    let params = use_params_map();
    let article = Resource::new(
        move || params().get("slug").unwrap_or_default(),
        |slug| async { get_article(slug).await },
    );

    let title = RwSignal::new(String::from("Loading"));

    view! {
        <Title text=move || title() />

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
                                title.set(article_result.article.slug.to_string());
                                view! { <ArticlePage username result=article_result /> }
                            })
                        })
                }}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
fn ArticlePage(username: crate::auth::UsernameSignal, result: ArticleResult) -> impl IntoView {
    let article_signal = RwSignal::new(result.article.clone());
    let user_signal = RwSignal::new(result.logged_user);

    let show_modal: RwSignal<bool> = use_context().expect("show_modal context should be available");
    show_modal.set(true);

    let global_state = expect_context::<Store<GlobalState>>();

    let on_back_event = move || {
        show_modal.set(false);
        let navigate = leptos_router::hooks::use_navigate();
        let url_str = global_state.back_url().get().to_string();
        navigate(&url_str, Default::default());
    };

    let following_signal =
        RwSignal::new(FollowUser(article_signal.get_untracked().author.following));
    provide_context(following_signal);

    view! {
        <Show when=move || show_modal.get()>
            <ArticlePageModal on_back_event username article_signal user_signal />
        </Show>
    }
}

#[component]
pub fn ArticlePageModal<C>(
    on_back_event: C,
    username: crate::auth::UsernameSignal,
    article_signal: ArticleSignal,
    user_signal: RwSignal<Option<User>>,
) -> impl IntoView
where
    C: Fn() + 'static + Copy,
{
    view! {
        <div class="bg-opacity-60 inset-0 z-50 flex items-center justify-center">
            <div class="z-70 block w-4/5 rounded-lg bg-white dark:bg-gray-800 dark:text-gray-100 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)]">
                <div class="mb-5 px-1 py-1">
                    <div class="mb-5">
                        <ArticleMeta username article=article_signal is_preview=false />
                    </div>
                    <div class="flex justify-between mb-5">
                        <div>
                            <div class="mb-2">
                                <h1 class="text-xl leading-tight font-medium text-neutral-800 dark:text-neutral-100">
                                    {article_signal.get_untracked().title}
                                </h1>
                            </div>
                            <AuthorUserIcon article_signal />
                        </div>
                        <div>
                            <BackToButton on_back_event is_top=true />
                        </div>
                    </div>
                    <div class="mb-5">
                        <p>{article_signal.get_untracked().body}</p>
                    </div>
                </div>
                <div class="mb-5 px-1 py-1">
                    <CommentSection username article=article_signal user_signal />
                </div>
                <BackToButton on_back_event is_top=false />
            </div>
        </div>
    }
}

#[component]
fn BackToButton<C>(on_back_event: C, is_top: bool) -> impl IntoView
where
    C: Fn() + 'static + Clone,
{
    view! {
        <form>
            <div class="flex justify-end mb-5">
                <button
                    type="cancel"
                    class=move || {
                        format!(
                            "fixed bg-blue-700 hover:bg-blue-800 px-15 py-3 text-white font-semibold rounded-lg transition-colors duration-300 {}",
                            if is_top { "top-0 left-0" } else { "bottom-4 right-4" },
                        )
                    }
                    on:click=move |_| on_back_event()
                >
                    Back
                </button>

            </div>
        </form>
    }
}

#[server(PostCommentAction, "/api")]
#[tracing::instrument]
pub async fn post_comment(slug: String, body: String) -> Result<(), ServerFnError> {
    let Some(logged_user) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Comment::insert(slug, logged_user, body)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post a comment, try again later".into())
        })
}

#[server(GetCommentsAction, "/api", "GetJson")]
#[tracing::instrument]
pub async fn get_comments(slug: String) -> Result<Vec<crate::models::Comment>, ServerFnError> {
    crate::models::Comment::get_all(slug).await.map_err(|x| {
        let err = format!("Error while posting a comment: {x:?}");
        tracing::error!("{err}");
        ServerFnError::ServerError("Could not post a comment, try again later".into())
    })
}

#[server(DeleteCommentsAction, "/api")]
#[tracing::instrument]
pub async fn delete_comment(id: i32) -> Result<(), ServerFnError> {
    let Some(logged_user) = crate::auth::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    crate::models::Comment::delete(id, logged_user)
        .await
        .map(|_| ())
        .map_err(|x| {
            let err = format!("Error while posting a comment: {x:?}");
            tracing::error!("{err}");
            ServerFnError::ServerError("Could not post a comment, try again later".into())
        })
}

#[component]
pub fn CommentSection(
    username: crate::auth::UsernameSignal,
    article: ArticleSignal,
    user_signal: RwSignal<Option<crate::models::User>>,
) -> impl IntoView {
    let comments_action = ServerAction::<PostCommentAction>::new();
    let result = comments_action.version();
    let comment: NodeRef<Textarea> = NodeRef::new();
    let (comment_value, set_comment_value) = signal(String::new());

    let on_comment_input = move |ev| {
        set_comment_value(event_target_value(&ev));
    };

    let comments = Resource::new(
        move || (result.get(), article.with(|a| a.slug.to_string())),
        move |(_, a)| async move {
            set_comment_value.set("".to_string());
            let comments = get_comments(a).await;
            let comments_count = comments
                .as_ref()
                .map(|c| c.len() as i64)
                .unwrap_or_default();
            article.update(|a| {
                a.comments_count = comments_count;
            });
            comments
        },
    );

    let post_button_disable =
        move || comment_value.get().is_empty() || comment_value.get().len() < 3;

    view! {
        <div class="mb-1">
            <Show when=move || username.with(Option::is_some) fallback=|| ()>
                <ActionForm action=comments_action>
                    <input
                        name="slug"
                        type="hidden"
                        value=move || article.with(|x| x.slug.to_string())
                    />
                    <h2 class="mb-2 block text-sm font-bold text-gray-700 dark:text-gray-300">Comments</h2>
                    <div class="mb-1">
                        <textarea
                            node_ref=comment
                            class="focus:shadow-outline w-full border-b dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 appearance-none rounded px-3 py-2 leading-tight text-sm text-gray-700 shadow focus:outline-none"
                            name="body"
                            prop:value=move || comment_value.get()
                            placeholder="Write a new comment...(min length 3 char)"
                            on:input=on_comment_input
                        ></textarea>
                    </div>
                    <div class="flex mb-5">
                        <CurrentUserIcon user_signal />
                        <div class="px-2">
                            <button
                                class=move || {
                                    format!(
                                        "rounded px-1 py-1 text-sm font-medium text-white {}",
                                        if post_button_disable() {
                                            "bg-gray-300 cursor-not-allowed"
                                        } else {
                                            "bg-blue-700 hover:bg-blue-800"
                                        },
                                    )
                                }
                                type="submit"
                                prop:disabled=move || post_button_disable
                            >
                                "Post Comment"
                            </button>
                        </div>
                    </div>
                </ActionForm>
            </Show>
            <Suspense fallback=move || view! { <p>"Loading Comments from the article"</p> }>
                <ErrorBoundary fallback=|_| {
                    view! { <p class="error-messages text-xs-center">"Something went wrong."</p> }
                }>
                    {move || {
                        comments
                            .get()
                            .map(move |x| {
                                x.map(move |c| {
                                    view! {
                                        <For
                                            each=move || c.clone().into_iter().enumerate()
                                            key=|(i, _)| *i
                                            children=move |(_, comment)| {
                                                let comment = RwSignal::new(comment);
                                                view! { <Comment username comment comments /> }
                                            }
                                        />
                                    }
                                })
                            })
                    }}
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}

#[component]
fn Comment(
    username: crate::auth::UsernameSignal,
    comment: RwSignal<crate::models::Comment>,
    comments: Resource<Result<Vec<Comment>, ServerFnError>>,
) -> impl IntoView {
    let delete_c = ServerAction::<DeleteCommentsAction>::new();
    let delete_result = delete_c.value();

    Effect::new(move |_| {
        if let Some(Ok(())) = delete_result.get() {
            tracing::info!("comment deleted!");
            comments.refetch();
        }
    });

    let comment_owner = username.get_untracked().unwrap_or_default()
        == comment.with_untracked(|x| x.username.to_string());

    view! {
        <div class="py-5">
            <CommentUserIcon comment />
            <div class="flex grow justify-between">
                <p>{move || comment.with(|x| x.body.to_string())}</p>
                <div class="flex-none px-3 text-gray-600">
                    <div>
                        <i class="fa-solid fa-calendar w-4 h-4"></i>
                        <span class="px-1">
                            {move || comment.with(|x| x.created_at.to_string())}
                        </span>
                    </div>
                    <Show when=move || comment_owner fallback=|| ()>
                        <div>
                            <ActionForm action=delete_c>
                                <input
                                    type="hidden"
                                    name="id"
                                    value=move || comment.with(|x| x.id)
                                />
                                <button
                                    class="text-red-400 hover:rounded hover:border hover:bg-red-100"
                                    type="submit"
                                >
                                    <i class="fas fa-trash"></i>
                                    <span class="px-1">Delete</span>
                                </button>
                            </ActionForm>
                        </div>
                    </Show>
                </div>
            </div>
        </div>
    }
}
