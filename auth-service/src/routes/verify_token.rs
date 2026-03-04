use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{app_state::AppState, domain::AuthAPIError, utils::auth::*};

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
#[derive(Deserialize)]
pub struct VerifyTokenResponse {
    pub token: String,
}

pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let token = request.token;

    if token.is_empty() {
        return Err(AuthAPIError::MissingToken);
    }

    let response = match validate_token(&token, state.banned_token_store).await {
        Ok(claims) => Ok((StatusCode::OK, claims.sub)),
        Err(_) => Err(AuthAPIError::InvalidToken),
    };

    response
}
