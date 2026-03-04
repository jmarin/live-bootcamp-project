use auth_service::{
    utils::{auth, constants::JWT_COOKIE_NAME},
    ErrorResponse,
};

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let invalid_inputs = [serde_json::json!({}), serde_json::json!({"token": 24})];

    for input in invalid_inputs.iter() {
        let response = app.post_verify_token(&input).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}

#[tokio::test]
async fn should_return_400_with_empty_token() {
    let app = TestApp::new().await;

    let token = serde_json::json!({"token": ""});

    let response = app.post_verify_token(&token).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({ "email": random_email, "password": "password123", "requires2FA": false });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    let token = auth_cookie.value();

    assert!(!token.is_empty());

    let verify_token_body = serde_json::json!({ "token": &token});

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    let token = serde_json::json!({"token": "bad_token"});

    let response = app.post_verify_token(&token).await;

    assert_eq!(response.status().as_u16(), 401);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid auth token".to_owned()
    );
}

#[tokio::test]
async fn should_return_401_banned_token() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({ "email": random_email, "password": "password123", "requires2FA": false });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let token = auth_cookie.value();

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 200);

    let verify_token_body = serde_json::json!({
        "token": token,
    });

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status().as_u16(), 401);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid auth token".to_owned()
    );
}
