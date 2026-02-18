use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = TestApp::get_random_email();

    let test_cases = [
        serde_json::json!({"password": "password123", "requires_2fa": true }),
        serde_json::json!({"email": random_email, "requires_2fa": true }),
        serde_json::json!({"requires_2fa": true }),
        serde_json::json!({"password": "password123"}),
        serde_json::json!({"email":  random_email}),
        serde_json::json!({}),
        serde_json::json!({"email": random_email, "password": "password123", "requires_2fa": false }), // this fails because requires_2fa has been renamed to requires2FA in the SignupRequest struct, so the deserialization will fail
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        )
    }
}
