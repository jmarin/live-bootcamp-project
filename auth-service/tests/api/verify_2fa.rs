use auth_service::{
    domain::{Email, LoginAttemptId, TwoFACode},
    routes::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
    ErrorResponse,
};

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let invalid_inputs = [
        serde_json::json!({}),
        serde_json::json!({"email": 23}),
        serde_json::json!({"email": "test@example.com", "password": "1234567"}),
    ];

    for input in invalid_inputs.iter() {
        let response = app.post_verify_2fa(&input).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let email = TestApp::get_random_email();
    let login_attempt_id = LoginAttemptId::default().as_ref().to_owned();
    let two_fa_code = TwoFACode::default().as_ref().to_owned();

    let invalid_inputs = [
        serde_json::json!({"email": "example.com", "loginAttemptId": login_attempt_id, "2FACode": two_fa_code}),
        serde_json::json!({"email": email, "loginAttemptId": "123", "2FACode": "123456"}),
        serde_json::json!({"email": email, "loginAttemptId": "abc", "2FACode": "123456"}),
    ];

    for input in invalid_inputs.iter() {
        let response = app.post_verify_2fa(&input).await;
        assert_eq!(response.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true,
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactoryAuthResponse");

    assert_eq!(response_body.message, "2FA required");
    assert!(!response_body.login_attempt_id.is_empty());

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(random_email.clone()).unwrap())
        .await
        .unwrap();

    let two_fa_code = code_tuple.1.as_ref();

    let incorrect_email = TestApp::get_random_email();
    let incorrect_login_attempt_id = LoginAttemptId::default().as_ref().to_owned();
    let incorrect_two_fa_code = TwoFACode::default().as_ref().to_owned();

    let input_values = vec![
        (
            incorrect_email.as_str(),
            login_attempt_id.as_str(),
            two_fa_code,
        ),
        (
            random_email.as_str(),
            incorrect_login_attempt_id.as_str(),
            two_fa_code,
        ),
        (
            random_email.as_str(),
            login_attempt_id.as_str(),
            incorrect_two_fa_code.as_ref(),
        ),
    ];

    for (email, login_attempt_id, two_fa_code) in input_values {
        let request_body = serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id,
            "2FACode": two_fa_code,
        });

        let response = app.post_verify_2fa(&request_body).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "Failed for input: {:?}",
            request_body
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response tp TwoFactorAuthResponse")
                .error,
            "Incorrect credentials".to_owned()
        )
    }
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    // Call login twice. Then, attempt to call verify-fa with the 2FA code from the first login requet. This should fail.
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true,
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactoryAuthResponse");

    assert_eq!(response_body.message, "2FA required");
    assert!(!response_body.login_attempt_id.is_empty());

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(random_email.clone()).unwrap())
        .await
        .unwrap();

    let two_fa_code = code_tuple.1.as_ref();

    let _ = app.post_login(&login_body).await;

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code,
    });
    let response = app.post_verify_2fa(&request_body).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    // Make sure to assert the auth cookie gets set

    let app = TestApp::new().await;
    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true,
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactoryAuthResponse");

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(random_email.clone()).unwrap())
        .await
        .unwrap();

    let two_fa_code = code_tuple.1.as_ref();

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code,
    });

    let response = app.post_verify_2fa(&request_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let app = TestApp::new().await;
    let random_email = TestApp::get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true,
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 206);

    let response_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactoryAuthResponse");

    let login_attempt_id = response_body.login_attempt_id;

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(random_email.clone()).unwrap())
        .await
        .unwrap();

    let two_fa_code = code_tuple.1.as_ref();

    let request_body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code,
    });

    let response = app.post_verify_2fa(&request_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let response = app.post_verify_2fa(&request_body).await;
    assert_eq!(response.status().as_u16(), 401);
}
