use auth_service::Application;
use uuid::Uuid;

//TestApp is test helper that is responsible for configuring/launching the auth service and providing methods for sending HTTP requests to the auth service.
pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let app = Application::build("127.0.0.1:0")
            .await
            .expect("Failed to build application");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        let _ = tokio::spawn(app.run());

        let http_client = 
         // Create a Reqwest http client instance
            reqwest::Client::builder()
                .build()
                .expect("Failed to build HTTP client");

        // Create new 'TestApp' instance and return it
        Self {address, http_client}
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response 
    where 
      Body: serde::Serialize
     {
        self.http_client
            .post(format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
     } 

    pub async fn post_login(&self) -> reqwest::Response {
        self.http_client
          .post(format!("{}/login", &self.address))
          .send()
          .await
          .expect("Failed to execute request")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn post_verify_token(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn get_random_email() -> String {
        format!("{}@example.com", Uuid::new_v4())
    } 
}
