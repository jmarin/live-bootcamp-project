use test_helpers::api_test;

use crate::helpers::TestApp;

use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::Url;

#[api_test]
async fn should_return_400_if_jwt_cookie_missing() {
    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);
}

#[api_test]
async fn should_return_401_if_invalid_token() {
    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 401);
}

#[api_test]
async fn should_return_200_if_valid_jwt_cookie() {
    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 200);

    let cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Cookie not found");

    assert!(!cookie.value().is_empty());

    let token = cookie.value();

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);

    let cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Cookie not found");

    assert!(!cookie.value().is_empty());

    let banned_token_store = app.banned_token_store.read().await;
    let contains_token = banned_token_store
        .contains_token(token)
        .await
        .expect("Failed to check banned token");
    assert!(contains_token);
    drop(banned_token_store);
}

#[api_test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status().as_u16(), 200);

    let cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Cookie not found");

    assert!(!cookie.value().is_empty());

    let _ = cookie.value();

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);

    let cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("Cookie not found");

    assert!(!cookie.value().is_empty());

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);
}
