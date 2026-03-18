use crate::app::{GlobalState, GlobalStateStoreFields};
use crate::routes::article_modal::ArticleResult;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;
use reactive_stores::Store;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum EditorResponse {
    ValidationError(String),
    UpdateError,
    Successful(String),
}
#[allow(dead_code)]
#[cfg_attr(feature = "hydrate", allow(dead_code))]
#[derive(Debug)]
struct ArticleUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: std::collections::HashSet<String>,
}

const TITLE_MIN_LENGTH: usize = 4;
const DESCRIPTION_MIN_LENGTH: usize = 4;
const BODY_MIN_LENGTH: usize = 10;

#[cfg(feature = "ssr")]
#[tracing::instrument]
fn validate_article(
    title: String,
    description: String,
    body: String,
    tag_list: String,
) -> Result<ArticleUpdate, String> {
    if title.len() < TITLE_MIN_LENGTH {
        return Err("You need to provide a title with at least 4 characters".into());
    }

    if description.len() < DESCRIPTION_MIN_LENGTH {
        return Err("You need to provide a description with at least 4 characters".into());
    }

    if body.len() < BODY_MIN_LENGTH {
        return Err("You need to provide a body with at least 10 characters".into());
    }

    let tag_list = tag_list
        .trim()
        .split_ascii_whitespace()
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect::<std::collections::HashSet<String>>();
    Ok(ArticleUpdate {
        title,
        description,
        body,
        tag_list,
    })
}

#[cfg(feature = "ssr")]
#[tracing::instrument]
async fn update_article(
    author: String,
    slug: String,
    article: ArticleUpdate,
) -> Result<String, sqlx::Error> {
    static BIND_LIMIT: usize = 65535;
    let mut transaction = crate::database::get_db().begin().await?;
    let (rows_affected, slug) = if !slug.is_empty() {
        (
            sqlx::query!(
                "UPDATE Articles SET title=$1, description=$2, body=$3 WHERE slug=$4 and author=$5",
                article.title,
                article.description,
                article.body,
                slug,
                author,
            )
            .execute(transaction.as_mut())
            .await?
            .rows_affected(),
            slug.to_string(),
        )
    } else {
        let slug = uuid::Uuid::now_v7().to_string();
        (sqlx::query!(
            "INSERT INTO Articles(slug, title, description, body, author) VALUES ($1, $2, $3, $4, $5)",
            slug,
            article.title,
            article.description,
            article.body,
            author
        )
        .execute(transaction.as_mut())
        .await?.rows_affected(),
        slug)
    };
    if rows_affected != 1 {
        // We are going to modify just one row, otherwise something funky is going on
        tracing::error!("no rows affected");
        return Err(sqlx::Error::RowNotFound);
    }
    sqlx::query!("DELETE FROM ArticleTags WHERE article=$1", slug)
        .execute(transaction.as_mut())
        .await?;
    if !article.tag_list.is_empty() {
        let mut qb = sqlx::QueryBuilder::new("INSERT INTO ArticleTags(article, tag) ");
        qb.push_values(
            article.tag_list.clone().into_iter().take(BIND_LIMIT / 2),
            |mut b, tag| {
                b.push_bind(slug.clone()).push_bind(tag);
            },
        );
        qb.build().execute(transaction.as_mut()).await?;
    }

    transaction.commit().await?;
    Ok(slug)
}

#[server(EditorAction, "/api")]
#[tracing::instrument]
pub async fn editor_action(
    title: String,
    description: String,
    body: String,
    tag_list: String,
    slug: String,
) -> Result<EditorResponse, ServerFnError> {
    let Some(author) = crate::auth::get_username() else {
        leptos_axum::redirect("/login");
        return Ok(EditorResponse::ValidationError(
            "you should be authenticated".to_string(),
        ));
    };
    let article = match validate_article(title, description, body, tag_list) {
        Ok(x) => x,
        Err(x) => return Ok(EditorResponse::ValidationError(x)),
    };
    match update_article(author, slug, article).await {
        Ok(x) => {
            // leptos_axum::redirect(&format!("/"));
            leptos_axum::redirect(&format!("/article/{x}"));
            Ok(EditorResponse::Successful(x))
        }
        Err(x) => {
            tracing::error!("EDITOR ERROR: {}", x.to_string());
            Ok(EditorResponse::UpdateError)
        }
    }
}

#[tracing::instrument]
#[component]
pub fn Editor() -> impl IntoView {
    let show_modal: RwSignal<bool> = use_context().expect("show_modal context should be available");
    show_modal.set(true);

    let (editor_status, set_editor_status) = signal(String::new());

    let editor_server_action = ServerAction::<EditorAction>::new();
    let result = editor_server_action.value();

    Effect::new(move || {
        editor_server_action.value().get();
        result.with(|msg| {
            msg.as_ref().map(|inner| match inner {
                Ok(EditorResponse::Successful(_slug)) => {
                    set_editor_status.set("Successful.".to_string());
                }

                Ok(EditorResponse::UpdateError) => {
                    set_editor_status
                        .set("Error while updating the article, please, try again later".into());
                }
                Ok(EditorResponse::ValidationError(x)) => {
                    set_editor_status.set(x.to_string());
                }
                Err(x) => {
                    set_editor_status.set(format!("Unexpected error: {x}"));
                }
            })
        });
    });

    let params = use_params_map();
    let article_res = OnceResource::new(async move {
        if let Some(s) = params.get_untracked().get("slug") {
            crate::routes::article_modal::get_article(s.to_string()).await
        } else {
            Ok(crate::routes::article_modal::ArticleResult::default())
        }
    });

    let on_submit_event = move |ea| {
        editor_server_action.dispatch(ea);
    };

    let global_state = expect_context::<Store<GlobalState>>();

    let on_cancel_event = move || {
        show_modal.set(false);
        editor_server_action.clear();
        let navigate = leptos_router::hooks::use_navigate();
        let url_str = global_state.back_url().get().to_string();
        navigate(&url_str, Default::default());
    };

    view! {
        <div>
            <EditorModal
                on_in=on_submit_event
                on_cancel=on_cancel_event
                editor_status
                article_res
            />
        </div>
    }
}

#[component]
fn EditorModal<A, C>(
    on_in: A,
    on_cancel: C,
    editor_status: ReadSignal<String>,
    article_res: OnceResource<Result<ArticleResult, ServerFnError>>,
) -> impl IntoView
where
    A: Fn(EditorAction) + 'static + Send + Copy,
    C: Fn() + 'static + Send + Copy,
{
    let editor_title: NodeRef<leptos::html::Input> = NodeRef::new();
    let editor_desc: NodeRef<leptos::html::Input> = NodeRef::new();
    let editor_body: NodeRef<leptos::html::Textarea> = NodeRef::new();
    let editor_tags: NodeRef<leptos::html::Input> = NodeRef::new();
    let editor_slug: NodeRef<leptos::html::Input> = NodeRef::new();

    let on_in_event = move |_| {
        let title = editor_title.get().expect("<input> to exist").value();
        let desc = editor_desc.get().expect("<input> to exist").value();
        let body = editor_body.get().expect("<textarea> to exist").value();
        let tags = editor_tags.get().expect("<input> to exist").value();
        let slug = editor_slug.get().expect("<input> to exist").value();

        on_in(EditorAction {
            title: title.clone(),
            description: desc,
            body,
            tag_list: tags,
            slug,
        })
    };
    view! {
        <Title text="Editor" />
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60">
            <div class="block rounded-lg bg-white dark:bg-gray-800 dark:text-gray-100 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70">
                <div>
                    <p class=move || {
                        format!(
                            "font-medium {}",
                            if editor_status.get() == "Successful." {
                                "text-gray-700 dark:text-gray-300"
                            } else {
                                "text-red-500"
                            },
                        )
                    }>
                        <strong>{move || editor_status.get()}</strong>

                    </p>

                    <div class="col-md-10 offset-md-1 col-xs-12">
                        <form>
                            <Suspense fallback=move || view! { <p>"Loading Tags"</p> }>
                                <ErrorBoundary fallback=|_| {
                                    view! {
                                        <p class="error-messages text-xs-center">
                                            "Something went wrong."
                                        </p>
                                    }
                                }>
                                    <div>
                                        <div class="mb-5">
                                            <input
                                                node_ref=editor_title
                                                name="title"
                                                type="text"
                                                class="input-field-common"
                                                minlength=TITLE_MIN_LENGTH
                                                placeholder="Article Title"
                                                value=move || {
                                                    article_res
                                                        .get()
                                                        .map(move |x| {
                                                            x.map(move |a| { a.article.title })
                                                                .unwrap_or("".to_string())
                                                        })
                                                        .unwrap_or_default()
                                                }
                                            />
                                        </div>
                                        <div class="mb-5">
                                            <input
                                                node_ref=editor_desc
                                                name="description"
                                                type="text"
                                                class="input-field-common"
                                                minlength=DESCRIPTION_MIN_LENGTH
                                                placeholder="What's this article about?"
                                                value=move || {
                                                    article_res
                                                        .get()
                                                        .map(move |x| {
                                                            x.map(move |a| { a.article.description })
                                                                .unwrap_or("".to_string())
                                                        })
                                                        .unwrap_or_default()
                                                }
                                            />
                                        </div>
                                        <div class="mb-5">
                                            <textarea
                                                node_ref=editor_body
                                                name="body"
                                                class="input-field-common"
                                                rows="8"
                                                placeholder="Write your article (in markdown)"
                                                minlength=BODY_MIN_LENGTH
                                                prop:value=move || {
                                                    article_res
                                                        .get()
                                                        .map(move |x| {
                                                            x.map(move |a| { a.article.body.unwrap_or_default() })
                                                                .unwrap_or("".to_string())
                                                        })
                                                        .unwrap_or_default()
                                                }
                                            ></textarea>
                                        </div>
                                        <div class="mb-5">
                                            <input
                                                node_ref=editor_tags
                                                name="tag_list"
                                                type="text"
                                                class="input-field-common"
                                                placeholder="Enter tags(space separated)"
                                                value=move || {
                                                    article_res
                                                        .get()
                                                        .map(move |x| {
                                                            x.map(move |a| { a.article.tag_list }).unwrap_or(Vec::new())
                                                        })
                                                        .unwrap_or_default()
                                                        .join(" ")
                                                }
                                            />
                                        </div>
                                        <div class="flex justify-between mb-5">
                                            <input
                                                node_ref=editor_slug
                                                name="slug"
                                                type="hidden"
                                                value=move || {
                                                    article_res
                                                        .get()
                                                        .map(move |x| {
                                                            x.map(move |a| { a.article.slug }).unwrap_or("".to_string())
                                                        })
                                                        .unwrap_or_default()
                                                }
                                            />
                                            <button
                                                class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg"
                                                type="button"
                                                on:click=on_in_event
                                            >
                                                "Publish Article"
                                            </button>
                                            <button
                                                type="cancel"
                                                class="bg-gray-300 hover:bg-gray-400 px-5 py-3 text-white rounded-lg"
                                                on:click=move |_| on_cancel()
                                            >
                                                Cancel
                                            </button>
                                        </div>
                                    </div>
                                </ErrorBoundary>
                            </Suspense>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    }
}
