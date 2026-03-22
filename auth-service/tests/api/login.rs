use auth_service::{
    domain::data_stores::LoginAttemptId, routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME, ErrorResponse,
};

use crate::helpers::TestApp;

use auth_service::domain::Email;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let test_cases = [
        serde_json::json!({"password": "password123"}),
        serde_json::json!({"email": random_email }),
        serde_json::json!({"password": "password123"}),
        serde_json::json!({"email":  random_email}),
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        )
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let invalid_inputs = [
        serde_json::json!({"email": "email.com", "password": "password123"}),
        serde_json::json!({"email": "email@example.com", "password": "pass"}),
        serde_json::json!({"email": "", "password": "password123"}),
    ];

    for input in invalid_inputs.iter() {
        let response = app.post_login(&input).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Invalid input: {:?}",
            input
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();
    let user = serde_json::json!(
        {
            "email": random_email,
            "password": "password123",
        }
    );

    let response = app.post_login(&user).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

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
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({
     "email": random_email,
     "password": "password123",
     "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    // TODO: assert that `json_body.login_attempt_id` is stored inside `app.two_fa_code_store`

    let login_attempt = json_body.login_attempt_id;
    let email = Email::parse(random_email).expect("Could not parse email");

    let read_2fa_code = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .expect("Could not read 2FA code from store");

    let login_attempt_id =
        LoginAttemptId::parse(login_attempt).expect("Could not parse LoginAttemptId");
    assert_eq!(read_2fa_code.0, login_attempt_id)
}
