use crate::app::{GlobalState, GlobalStateStoreFields};
use crate::auth::LogoutAction;
use leptos::html::Input;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{hooks::use_query, params::Params};
use reactive_stores::Store;
use std::env;

#[allow(dead_code)]
#[cfg(feature = "ssr")]
struct EmailCredentials {
    email: String,
    passwd: String,
    smtp_server: String,
}

#[cfg(feature = "ssr")]
static EMAIL_CREDS: std::sync::OnceLock<EmailCredentials> = std::sync::OnceLock::new();

#[tracing::instrument]
#[server(ResetPasswordAction1, "/api")]
pub async fn reset_password_1(email: String) -> Result<String, ServerFnError> {
    if let Err(x) = crate::models::User::get_email(email.clone()).await {
        let err = format!("Bad email ID: Provided email not found.");
        tracing::error!("{err} {x:?} ");
        return Err(ServerFnError::new(err));
    } else {
        let creds = EMAIL_CREDS.get_or_init(|| EmailCredentials {
            email: env::var("MAILER_EMAIL").unwrap(),
            passwd: env::var("MAILER_PASSWD").unwrap(),
            smtp_server: env::var("MAILER_SMTP_SERVER").unwrap(),
        });
        let host = leptos_axum::extract::<axum_extra::typed_header::TypedHeader<headers::Host>>()
            .await?
            .0;
        let schema = if cfg!(debug_assertions) {
            "http"
        } else {
            "https"
        };
        let token = crate::auth::encode_token(crate::auth::TokenClaims {
            sub: email.clone(),
            exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + 3_600,
        })
        .unwrap();
        let uri = format!("{}://{}/reset_password?token={}", schema, host, token);
        // Build a simple multipart message
        let message = mail_send::mail_builder::MessageBuilder::new()
            .from(("Realworld Leptos", creds.email.as_str()))
            .to(vec![("You", email.as_str())])
            .subject("Your password reset from realworld leptos")
            .text_body(format!(
                "You can reset your password accessing the following link: {uri}"
            ));

        // Connect to the SMTP submissions port, upgrade to TLS and
        // authenticate using the provided credentials.
        leptos::logging::log!("The email is {:?}", message);

        // ********* UNCOMMENT IF NEEDED *********
        // if smtp available, then uncomment below mail send part. Else use above logging to get a reset link to test
        // Incorrect smtp may cause the thread to panic after multiple attempts

        // mail_send::SmtpClientBuilder::new(creds.smtp_server.as_str(), 587)
        //     .implicit_tls(false)
        //     .credentials((creds.email.as_str(), creds.passwd.as_str()))
        //     .connect()
        //     .await?
        //     .send(message)
        //     .await?
    }
    return Ok(String::from(
        "Email sent. Check email and click the reset url link inside.",
    ));
}

#[tracing::instrument]
#[server(ResetPasswordAction2, "/api")]
pub async fn reset_password_2(
    token: String,
    password: String,
    confirm: String,
) -> Result<String, ServerFnError> {
    if !(password == confirm) {
        return Err(ServerFnError::new(
            "Passwords do not match, please retry!".to_string(),
        ));
    }
    let Ok(claims) = crate::auth::decode_token(token.as_str()) else {
        tracing::info!("Invalid token provided");
        return Err(ServerFnError::new("Invalid token provided!".to_string()));
    };
    let email = claims.claims.sub;
    let Ok(user) = crate::models::User::get_email(email.clone()).await else {
        tracing::info!("User does not exist");
        return Err(ServerFnError::new("User does not exist!".to_string()));
    };
    match user.set_password(password) {
        Ok(u) => {
            if let Err(error) = u.update().await {
                tracing::error!(email, ?error, "error while resetting the password");
                return Err(ServerFnError::new(error.to_string()));
            } else {
                // A real password reset would have a list of issued tokens and invalidation over
                // the used ones. As this would grow much bigger in complexity, I prefer to write
                // down this security vulnerability and left it simple :)
                // message = String::from("Password successfully reset, please, proceed to login");
                return Ok("Password successfully changed, please, proceed to login".to_string());
            }
        }
        Err(x) => {
            return Err(ServerFnError::new(x));
        }
    }
}

#[derive(Params, PartialEq)]
struct TokenQuery {
    token: Option<String>,
}

#[component]
pub fn ResetPassword(logout: ServerAction<LogoutAction>) -> impl IntoView {
    let q = use_query::<TokenQuery>();

    let show_modal: RwSignal<bool> = use_context().expect("show_modal context should be available");
    show_modal.set(true);

    let reset_status = RwSignal::new(String::new());
    let email_or_password = RwSignal::new("email".to_string());

    let reset_password_action1 = ServerAction::<ResetPasswordAction1>::new();
    let result_1 = reset_password_action1.value();

    let reset_password_action2 = ServerAction::<ResetPasswordAction2>::new();
    let result_2 = reset_password_action2.value();

    Effect::new(move || {
        reset_password_action1.value().get();
        result_1.with(|msg| {
            msg.as_ref().map(|inner| match inner {
                Ok(msg) => {
                    reset_status.set(msg.to_string());
                }
                Err(err) => {
                    reset_status.set(err.to_string());
                }
            })
        });
    });

    Effect::new(move || {
        reset_password_action2.value().get();
        result_2.with(|msg| {
            msg.as_ref().map(|inner| match inner {
                Ok(msg) => {
                    reset_status.set(msg.to_string());
                    logout.dispatch(LogoutAction {});
                }
                Err(err) => reset_status.set(err.to_string()),
            })
        });
    });

    let email_in_event = move |eu| {
        reset_password_action1.dispatch(eu);
    };

    let passwd_in_event = move |pu| {
        reset_password_action2.dispatch(pu);
    };

    let global_state = expect_context::<Store<GlobalState>>();

    let cancel_event = move || {
        show_modal.set(false);
        reset_password_action1.clear();
        reset_password_action2.clear();
        let navigate = leptos_router::hooks::use_navigate();
        let url_str = global_state.back_url().get().to_string();
        navigate(&url_str, Default::default());
    };

    view! {
        <div>
            {move || {
                let mut token_string = String::new();
                q.with(|x| {
                    if let Ok(token_query) = x {
                        if let Some(token) = &token_query.token {
                            token_string = token.to_string();
                            email_or_password.set("passwd".to_string());
                        }
                    }
                });
                view! {
                    <ResetPasswordModal
                        email_in=email_in_event
                        passwd_in=passwd_in_event
                        on_cancel=cancel_event
                        reset_status
                        email_or_password
                        token=token_string
                    />
                }
            }}
        </div>
    }
}

#[component]
fn ResetPasswordModal<E, P, C>(
    email_in: E,
    passwd_in: P,
    on_cancel: C,
    reset_status: RwSignal<String>,
    email_or_password: RwSignal<String>,
    token: String,
) -> impl IntoView
where
    E: Fn(ResetPasswordAction1) + 'static + Send,
    P: Fn(ResetPasswordAction2) + 'static + Send,
    C: Fn() + 'static + Send,
{
    let user_email: NodeRef<Input> = NodeRef::new();
    let user_new_password: NodeRef<Input> = NodeRef::new();
    let user_confirm_password: NodeRef<Input> = NodeRef::new();

    let on_in_event = move |_| {
        if email_or_password.get() == "email" {
            let email = user_email.get().expect("email <Input> to exist").value();
            email_in(ResetPasswordAction1 { email })
        } else {
            let password = user_new_password
                .get()
                .expect("new password <Input> to exist")
                .value();
            let confirm = user_confirm_password
                .get()
                .expect("confirm password <Input> to exist")
                .value();
            passwd_in(ResetPasswordAction2 {
                token: token.clone(),
                password,
                confirm,
            })
        }
    };

    let (email_value, set_email_value) = signal(String::new());
    let (new_password_value, set_new_password_value) = signal(String::new());
    let (confirm_password_value, set_confirm_password_value) = signal(String::new());

    let on_email_input = move |ev| set_email_value(event_target_value(&ev));

    let passwords_dont_match = move || {
        !(new_password_value.get().is_empty() && confirm_password_value.get().is_empty())
            && new_password_value.get() != confirm_password_value.get()
    };

    let run_password_match_update = move || {
        if passwords_dont_match() {
            reset_status.set("Passwords do not match!".to_string())
        } else {
            reset_status.set(
                "Passwords matched!\n\n
                Proceed password reset followed by logout?"
                    .to_string(),
            )
        }
    };

    let on_new_password_input = move |ev| {
        set_new_password_value(event_target_value(&ev));
        run_password_match_update();
    };
    let on_confirm_password_input = move |ev| {
        set_confirm_password_value(event_target_value(&ev));
        run_password_match_update();
    };

    let is_button_disabled = move || {
        (email_value.get().is_empty() && email_or_password.get() == "email")
            || ((new_password_value.get().is_empty() && confirm_password_value.get().is_empty()
                || passwords_dont_match())
                && email_or_password.get() == "passwd")
            || reset_status.get().starts_with("Email sent.")
    };

    let cancel_button_string = move || {
        format!(
            "{}",
            if reset_status
                .get()
                .starts_with("Password successfully changed")
                || reset_status.get().starts_with("Email sent")
            {
                "Back to Home"
            } else {
                "Cancel"
            }
        )
    };
    let reset_button_string = move || {
        format!(
            "{}",
            if email_or_password.get() == "email" {
                "Send Reset link in email"
            } else {
                "Reset Password & Logout"
            }
        )
    };

    let (passwd_visible, set_passwd_visible) = signal(false);

    view! {
        <Title text="Reset Password" />
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60">

            <div class="block rounded-lg bg-white w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70">
                <h5 class="mb-5 text-xl font-medium leading-tight text-neutral-800">
                    Reset Password.
                </h5>
                <form>
                    <Show
                        when=move || email_or_password.get() == "email"
                        fallback=move || {
                            view! {
                                <label
                                    class="block text-gray-700 text-sm font-bold mb-2"
                                    for="password"
                                >
                                    Set a new password.

                                </label>
                                <div class="mb-5 relative">
                                    <input
                                        node_ref=user_new_password
                                        name="password"
                                        class="input-field-common"
                                        type=move || {
                                            format!(
                                                "{}",
                                                if passwd_visible.get() { "text" } else { "password" },
                                            )
                                        }
                                        placeholder="New password"
                                        value=move || { String::new() }
                                        on:input=on_new_password_input
                                    />
                                    <span
                                        class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer"
                                        on:click=move |_| {
                                            set_passwd_visible.set(!passwd_visible.get())
                                        }
                                    >
                                        <i class=move || {
                                            format!(
                                                "{}",
                                                if passwd_visible.get() {
                                                    "far fa-eye"
                                                } else {
                                                    "far fa-eye-slash"
                                                },
                                            )
                                        }></i>
                                    </span>
                                </div>
                                <div class="mb-5 relative">
                                    <input
                                        node_ref=user_confirm_password
                                        name="confirm_password"
                                        class="input-field-common"
                                        type=move || {
                                            format!(
                                                "{}",
                                                if passwd_visible.get() { "text" } else { "password" },
                                            )
                                        }
                                        placeholder="Confirm password"
                                        value=move || { String::new() }
                                        on:input=on_confirm_password_input
                                    />
                                    <span
                                        class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer"
                                        on:click=move |_| {
                                            set_passwd_visible.set(!passwd_visible.get())
                                        }
                                    >
                                        <i class=move || {
                                            format!(
                                                "{}",
                                                if passwd_visible.get() {
                                                    "far fa-eye"
                                                } else {
                                                    "far fa-eye-slash"
                                                },
                                            )
                                        }></i>
                                    </span>
                                </div>
                            }
                        }
                    >
                        <div class="mb-5">
                            <label class="block text-gray-700 text-sm font-bold mb-2" for="email">
                                Provide your linked email address with your user account.
                                <input
                                    node_ref=user_email
                                    class="input-field-common"
                                    id="email"
                                    name="email"
                                    type="text"
                                    value=move || { String::new() }
                                    placeholder="Registered email"
                                    required=true
                                    on:input=on_email_input
                                />
                            </label>
                        </div>
                    </Show>
                    <div class="mb-5">
                        <p class=move || {
                            format!(
                                "font-medium {}",
                                if reset_status.get().starts_with("Email sent")
                                    || reset_status
                                        .get()
                                        .starts_with("Password successfully changed")
                                    || reset_status.get().starts_with("Passwords matched!")
                                {
                                    "text-green-500"
                                } else {
                                    "text-red-500"
                                },
                            )
                        }>
                            <strong>{move || reset_status.get()}</strong>

                        </p>
                    </div>
                    <div class="flex justify-between mb-5">
                        <button
                            class="btn-primary"
                            type="button"
                            prop:disabled=move || is_button_disabled
                            on:click=on_in_event
                        >
                            {reset_button_string}
                        </button>
                        <button
                            type="cancel"
                            class=move || {
                                format!(
                                    "{}",
                                    if reset_status.get().starts_with("Email sent.") {
                                        "btn-primary"
                                    } else {
                                        "bg-gray-300 hover:bg-gray-400 px-5 py-3 text-white rounded-lg"
                                    },
                                )
                            }
                            on:click=move |_| on_cancel()
                        >
                            {cancel_button_string}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
