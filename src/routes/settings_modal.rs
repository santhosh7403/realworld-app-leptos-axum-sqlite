use crate::app::{GlobalState, GlobalStateStoreFields};
use leptos::{
    html::{Input, Textarea},
    prelude::*,
};
use leptos_meta::*;
use reactive_stores::Store;

use serde::{Deserialize, Serialize};

use crate::auth::LogoutAction;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum SettingsUpdateError {
    PasswordsNotMatch,
    Successful,
    ValidationError(String),
}

#[tracing::instrument]
#[server(SettingsUpdateAction, "/api")]
pub async fn settings_update(
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
) -> Result<SettingsUpdateError, ServerFnError> {
    let user = get_user().await?;
    let username = user.username();
    let user = match update_user_validation(user, image, bio, email, password, &confirm_password) {
        Ok(x) => x,
        Err(x) => return Ok(x),
    };
    user.update()
        .await
        .map(|_| SettingsUpdateError::Successful)
        .map_err(move |x| {
            tracing::error!(
                "Problem while updating user: {} with error {}",
                username,
                x.to_string()
            );
            ServerFnError::ServerError("Problem while updating user".into())
        })
}

#[cfg(feature = "ssr")]
fn update_user_validation(
    mut user: crate::models::User,
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: &str,
) -> Result<crate::models::User, SettingsUpdateError> {
    if !password.is_empty() {
        if password != confirm_password {
            return Err(SettingsUpdateError::PasswordsNotMatch);
        }
        user = user
            .set_password(password)
            .map_err(SettingsUpdateError::ValidationError)?;
    }

    user.set_email(email)
        .map_err(SettingsUpdateError::ValidationError)?
        .set_bio(bio)
        .map_err(SettingsUpdateError::ValidationError)?
        .set_image(image)
        .map_err(SettingsUpdateError::ValidationError)
}

#[cfg(feature = "ssr")]
async fn get_user() -> Result<crate::models::User, ServerFnError> {
    let Some(username) = crate::auth::get_username() else {
        leptos_axum::redirect("/login");
        return Err(ServerFnError::ServerError(
            "You need to be authenticated".to_string(),
        ));
    };

    crate::models::User::get(username).await.map_err(|x| {
        let err = x.to_string();
        tracing::error!("problem while getting the user {err}");
        ServerFnError::ServerError(err)
    })
}

#[tracing::instrument]
#[server(SettingsGetAction, "/api", "GetJson")]
pub async fn settings_get() -> Result<crate::models::User, ServerFnError> {
    get_user().await
}

// #[derive(Debug, Default, Deserialize, Serialize, Clone)]
// pub struct UserGet {
//     username: String,
//     email: String,
//     bio: Option<String>,
//     image: Option<String>,
// }

#[component]
pub fn Settings(logout: ServerAction<LogoutAction>) -> impl IntoView {
    let show_modal: RwSignal<bool> = use_context().expect("show_modal context should be available");
    show_modal.set(true);

    let update_status = RwSignal::new(String::new());
    let settings_server_action = ServerAction::<SettingsUpdateAction>::new();
    let result = settings_server_action.value();

    let resource = Resource::new(
        move || settings_server_action.version().get(),
        move |_| settings_get(),
    );

    let logout_signal = RwSignal::new(Some(false));

    Effect::new(move || {
        settings_server_action.value().get();
        result.with(|msg| {
            msg.as_ref().map(|inner| match inner {
                Ok(SettingsUpdateError::Successful) => {
                    update_status.set("Successful.".to_string());
                    if let Some(true) = logout_signal.get() {
                        logout.dispatch(LogoutAction {});
                    }
                }

                Ok(SettingsUpdateError::PasswordsNotMatch) => {
                    update_status
                        .set("Error: New password and Confirm password do not match!".into());
                }
                Ok(SettingsUpdateError::ValidationError(x)) => {
                    update_status.set(x.to_string());
                }
                Err(x) => {
                    update_status.set(format!("Unexpected error: {x}"));
                }
            })
        });
    });

    let settings_update_event = move |su| {
        settings_server_action.dispatch(su);
    };

    let settings_update_logout_event = move |su| {
        settings_server_action.dispatch(su);
        logout_signal.set(Some(true));
    };

    let global_state = expect_context::<Store<GlobalState>>();

    let on_cancel_event = move || {
        show_modal.set(false);
        settings_server_action.clear();
        let navigate = leptos_router::hooks::use_navigate();
        let url_str = global_state.back_url().get().to_string();
        navigate(&url_str, Default::default());
    };

    view! {
        <Show when=move || show_modal.get()>
            <Suspense fallback=move || view! { <p>"Loading user settings"</p> }>
                <ErrorBoundary fallback=|_| {
                    view! { <p>"There was a problem while fetching settings, try again later"</p> }
                }>
                    {move || {
                        resource
                            .get()
                            .map(move |x| {
                                x.map(move |user| {
                                    view! {
                                        <SettingsModal
                                            on_in=settings_update_event
                                            on_in_logout=settings_update_logout_event
                                            on_cancel=on_cancel_event
                                            user
                                            update_status
                                        />
                                    }
                                })
                            })
                    }}
                </ErrorBoundary>
            </Suspense>
        </Show>
    }
}

#[component]
fn SettingsModal<A, B, C>(
    on_in: A,
    on_in_logout: B,
    on_cancel: C,
    user: crate::models::User,
    update_status: RwSignal<String>,
) -> impl IntoView
where
    A: Fn(SettingsUpdateAction) + 'static + Send,
    B: Fn(SettingsUpdateAction) + 'static + Send,
    C: Fn() + 'static + Send,
{
    let user_profile_pic_url: NodeRef<Input> = NodeRef::new();
    let user_name: NodeRef<Input> = NodeRef::new();
    let user_bio: NodeRef<Textarea> = NodeRef::new();
    let user_email: NodeRef<Input> = NodeRef::new();
    let user_new_password: NodeRef<Input> = NodeRef::new();
    let user_confirm_password: NodeRef<Input> = NodeRef::new();

    let prev_data = RwSignal::new(user.clone());
    let no_profile_url_input_yet = RwSignal::new(true);
    let no_bio_input_yet = RwSignal::new(true);
    let no_email_input_yet = RwSignal::new(true);

    let (profile_pic_url_value, set_profile_pic_url_value) = signal(String::new());
    let (bio_value, set_bio_value) = signal(String::new());
    let (email_value, set_email_value) = signal(String::new());
    let (new_password_value, set_new_password_value) = signal(String::new());
    let (confirm_password_value, set_confirm_password_value) = signal(String::new());

    let on_profile_pic_url_input = move |ev| {
        set_profile_pic_url_value(event_target_value(&ev));
        no_profile_url_input_yet.set(false);
    };
    let on_bio_input = move |ev| {
        set_bio_value(event_target_value(&ev));
        no_bio_input_yet.set(false);
    };
    let on_email_input = move |ev| {
        set_email_value(event_target_value(&ev));
        no_email_input_yet.set(false);
    };

    let passwords_dont_match = move || {
        !(new_password_value.get().is_empty() && confirm_password_value.get().is_empty())
            && new_password_value.get() != confirm_password_value.get()
    };

    let passwords_empty = move || {
        new_password_value.get().trim().is_empty() && confirm_password_value.get().trim().is_empty()
    };

    let run_password_match_update = move || {
        if passwords_dont_match() {
            update_status.set("Passwords do not match!".to_string())
        } else if !passwords_empty() {
            update_status.set(
                "Passwords matched!\n\n
                Proceed password reset followed by logout?"
                    .to_string(),
            )
        } else {
            update_status.set("".to_string())
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

    let prev_non_empty_fields_now_empty = move || {
        let mut counter = vec![];
        counter.push(
            profile_pic_url_value.get().is_empty()
                && !no_profile_url_input_yet.get()
                && prev_data.get().image().is_some(),
        );
        counter.push(
            bio_value.get().trim().is_empty()
                && !no_bio_input_yet.get()
                && prev_data.get().bio().is_some(),
        );
        counter.push(
            email_value.get().is_empty()
                && !no_email_input_yet.get()
                && prev_data.get().email().is_empty(),
        );

        counter.iter().filter(|b| **b).collect::<Vec<_>>().len()
    };

    let fields_not_eqls_prev_non_empty = move || {
        let mut counter = vec![];
        counter.push(
            prev_data.get().image() != Some(profile_pic_url_value.get())
                && !no_profile_url_input_yet.get()
                && prev_data.get().image().is_some(),
        );
        counter.push(
            prev_data.get().bio() != Some(bio_value.get())
                && !no_bio_input_yet.get()
                && prev_data.get().bio().is_some(),
        );
        counter.push(
            prev_data.get().email() != email_value.get()
                && !no_email_input_yet.get()
                && !prev_data.get().email().is_empty(),
        );

        counter.iter().filter(|b| **b).collect::<Vec<_>>().len()
    };
    let fields_not_eqls_prev_empty = move || {
        let mut counter = vec![];
        counter.push(
            !profile_pic_url_value.get().is_empty()
                && !no_profile_url_input_yet.get()
                && prev_data.get().image().is_none(),
        );
        counter.push(
            !bio_value.get().is_empty()
                && !no_bio_input_yet.get()
                && prev_data.get().bio().is_none(),
        );
        counter.push(
            !email_value.get().is_empty()
                && !no_email_input_yet.get()
                && prev_data.get().email().is_empty(),
        );

        counter.iter().filter(|b| **b).collect::<Vec<_>>().len()
    };

    // Ref: https://book.leptos.dev/view/05_forms.html#controlled-inputs
    //
    //  All fields will be blank after a page load till on:input

    let blank_input_fields = move || {
        let mut counter = vec![];
        counter.push(no_profile_url_input_yet.get());
        counter.push(no_bio_input_yet.get());
        counter.push(no_email_input_yet.get());

        counter.iter().filter(|b| **b).collect::<Vec<_>>().len()
    };

    const TOT_FIELDS: usize = 3;

    let is_button_disabled = move || {
        (blank_input_fields() == TOT_FIELDS && passwords_empty())
            || (prev_non_empty_fields_now_empty() == 0
                && fields_not_eqls_prev_empty() == 0
                && fields_not_eqls_prev_non_empty() == 0)
                && passwords_empty()
    };

    let on_in_event = move |_| {
        update_status.set("".to_string());
        let profile_pic_url = user_profile_pic_url
            .get()
            .expect("profile pic url <input> to exist")
            .value();

        let bio = user_bio.get().expect("bio <textarea> to exist").value();
        let email = user_email.get().expect("email <input> to exist").value();
        let new_password = user_new_password
            .get()
            .expect("new_password <input> to exist")
            .value();
        let confirm_password = user_confirm_password
            .get()
            .expect("confirm_password <input> to exist")
            .value();
        if !passwords_dont_match() && !passwords_empty() {
            on_in_logout(SettingsUpdateAction {
                image: profile_pic_url,
                bio,
                email,
                password: new_password,
                confirm_password,
            })
        } else {
            on_in(SettingsUpdateAction {
                image: profile_pic_url,
                bio,
                email,
                password: new_password,
                confirm_password,
            })
        }
    };

    let update_button_string = move || {
        format!(
            "{}",
            if !passwords_dont_match() && !passwords_empty() {
                "Reset Password & Logout"
            } else {
                "Update Settings"
            }
        )
    };
    let cancel_button_string = move || {
        format!(
            "{}",
            if update_status.get().starts_with("Successful.") {
                "Back to Home"
            } else {
                "Cancel"
            }
        )
    };

    let (passwd_visible, set_passwd_visible) = signal(false);

    view! {
        <Title text="Settings" />
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60">
            <div class="block rounded-lg bg-white dark:bg-gray-800 dark:text-gray-100 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70">
                <h5 class="mb-5 text-xl font-medium leading-tight text-neutral-500 dark:text-neutral-400">
                    Update Your Settings.
                </h5>

                <form>
                    <div class="mb-5">
                        <input
                            node_ref=user_profile_pic_url
                            class="input-field-common"
                            name="username"
                            type="text"
                            value=user.image()
                            placeholder="URL of profile picture"
                            on:input=on_profile_pic_url_input
                        />
                    </div>
                    <div class="mb-5">
                        <input
                            disabled
                            node_ref=user_name
                            class="input-field-common cursor-not-allowed"
                            type="text"
                            placeholder=user.username()
                        />
                    </div>
                    <div class="mb-5">
                        <textarea
                            name="bio"
                            node_ref=user_bio
                            class="input-field-common"
                            prop:value=user.bio().unwrap_or_default()
                            placeholder="Short bio about you"
                            on:input=on_bio_input
                        />
                    </div>
                    <div class="mb-5">
                        <input
                            node_ref=user_email
                            class="input-field-common"
                            value=user.email()
                            type="text"
                            placeholder="Email (required)"
                            on:input=on_email_input
                        />
                    </div>
                    <div class="mb-5 relative">
                        <input
                            node_ref=user_new_password
                            name="password"
                            class="input-field-common"
                            // type="password"
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
                            on:click=move |_| { set_passwd_visible.set(!passwd_visible.get()) }
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
                            on:click=move |_| { set_passwd_visible.set(!passwd_visible.get()) }
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
                    <div class="mb-5">
                        <p class=move || {
                            format!(
                                "font-medium {}",
                                if update_status.get().starts_with("Successful.")
                                    || update_status.get().starts_with("Passwords matched")
                                {
                                    "text-green-500"
                                } else {
                                    "text-red-500"
                                },
                            )
                        }>
                            <strong>{move || update_status.get()}</strong>

                        </p>
                    </div>
                    <div class="flex justify-between mb-5">
                        <button
                            class="btn-primary"
                            type="button"
                            prop:disabled=move || is_button_disabled
                            on:click=on_in_event
                        >
                            {update_button_string}
                        </button>
                        <button
                            type="cancel"
                            class=move || {
                                format!(
                                    "{}",
                                    if update_status.get().starts_with("Successful.") {
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
