use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie, CookieJar};

use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(c) => c,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value().to_owned();

    let banned_token_store = state.banned_token_store.clone();

    let _ = match validate_token(&token, banned_token_store.clone()).await {
        Ok(claims) => claims,
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken)),
    };

    let mut banned_token_store_write = banned_token_store.write().await;
    let _ = banned_token_store_write.add_token(&token).await;

    let jar = jar.remove(cookie::Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
