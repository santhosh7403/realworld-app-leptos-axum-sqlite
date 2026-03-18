#[cfg(feature = "ssr")]
use argon2::{password_hash::PasswordVerifier, Argon2};
use leptos::prelude::*;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum SignupResponse {
    ValidationError(String),
    CreateUserError(String),
    Success,
}

#[tracing::instrument]
pub fn validate_signup(
    username: String,
    email: String,
    password: String,
) -> Result<crate::models::User, String> {
    crate::models::User::default()
        .set_username(username)?
        .set_password(password)?
        .set_email(email)
}

#[tracing::instrument]
#[server(SignupAction, "/api")]
pub async fn signup_action(
    username: String,
    email: String,
    password: String,
) -> Result<SignupResponse, ServerFnError> {
    match validate_signup(username.clone(), email, password) {
        Ok(user) => match user.insert().await {
            Ok(_) => {
                crate::auth::set_username(username).await;
                // leptos_axum::redirect("/");
                Ok(SignupResponse::Success)
            }
            Err(x) => {
                let x = x.to_string();
                Ok(if x.contains("UNIQUE constraint failed: Users.email") {
                    SignupResponse::CreateUserError("Duplicated email".to_string())
                } else if x.contains("UNIQUE constraint failed: Users.username") {
                    SignupResponse::CreateUserError("Duplicated user".to_string())
                } else {
                    tracing::error!("error from DB: {}", x);
                    SignupResponse::CreateUserError(
                        "There is some problem in user creation, check log".to_string(),
                    )
                })
            }
        },
        Err(x) => Ok(SignupResponse::ValidationError(x)),
    }
}

#[server(LoginAction, "/api")]
#[tracing::instrument]
pub async fn login_action(username: String, password: String) -> Result<String, ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>().unwrap();

    let hash_pass_row = sqlx::query!("SELECT password FROM Users where username=$1", username)
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|err| {
            tracing::debug!("DB err: {}", err);
            response_options.set_status(axum::http::StatusCode::FORBIDDEN);
            ServerFnError::new("Unsuccessful: User not available".to_string())
        })?;

    let parsed_hash =
        argon2::password_hash::PasswordHash::new(&hash_pass_row.password).map_err(|_| {
            response_options.set_status(axum::http::StatusCode::FORBIDDEN);
            ServerFnError::new("Unsuccessful: Hash error".to_string())
        })?;

    let argon2 = Argon2::default();
    if argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        crate::auth::set_username(username).await;
        leptos_axum::redirect("/");
        Ok("Successful".to_string())
    } else {
        response_options.set_status(axum::http::StatusCode::FORBIDDEN);
        Err(ServerFnError::new(
            "Unsuccessful: Password not matching".to_string(),
        ))
    }
}

#[server(LogoutAction, "/api")]
#[tracing::instrument]
pub async fn logout_action() -> Result<(), ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>().unwrap();
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        axum::http::HeaderValue::from_str(crate::auth::REMOVE_COOKIE)
            .expect("header value couldn't be set"),
    );
    leptos_axum::redirect("/");
    // leptos_axum::redirect("/login");
    Ok(())
}

#[server(CurrentUserAction, "/api")]
#[tracing::instrument]
pub async fn current_user() -> Result<crate::models::User, ServerFnError> {
    let Some(logged_user) = super::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };
    crate::models::User::get(logged_user).await.map_err(|err| {
        tracing::error!("problem while retrieving current_user: {err:?}");
        ServerFnError::ServerError("you must be logged in".into())
    })
}

#[server(UpdateThemeMode, "/api")]
#[tracing::instrument]
pub async fn update_theme_mode(theme: String) -> Result<(), ServerFnError> {
    let Some(logged_user) = super::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    match crate::models::User::get(logged_user).await {
        Ok(user) => {
            if let Err(err) = user.set_theme_mode(theme).update_theme_mode().await {
                tracing::error!("problem while updating theme_mode: {err:?}");
                Err(ServerFnError::ServerError("failed to update theme".into()))
            } else {
                Ok(())
            }
        }
        Err(err) => {
            tracing::error!("problem while retrieving current_user: {err:?}");
            Err(ServerFnError::ServerError("you must be logged in".into()))
        }
    }
}

#[server(UpdatePerPageAmount, "/api")]
#[tracing::instrument]
pub async fn update_per_page_amount(amount: u32) -> Result<(), ServerFnError> {
    let Some(logged_user) = super::get_username() else {
        return Err(ServerFnError::ServerError("you must be logged in".into()));
    };

    match crate::models::User::get(logged_user).await {
        Ok(user) => {
            if let Err(err) = user
                .set_per_page_amount(amount as i64)
                .update_per_page_amount()
                .await
            {
                tracing::error!("problem while updating per_page_amount: {err:?}");
                Err(ServerFnError::ServerError(
                    "failed to update per page amount".into(),
                ))
            } else {
                Ok(())
            }
        }
        Err(err) => {
            tracing::error!("problem while retrieving current_user: {err:?}");
            Err(ServerFnError::ServerError("you must be logged in".into()))
        }
    }
}
