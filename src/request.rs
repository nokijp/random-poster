use serde::Serialize;

#[derive(PartialEq, Eq, Serialize, Debug)]
pub struct SimpleWebhookRequest {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub content: String,
}

pub async fn post(webhook_url: &str, request: &SimpleWebhookRequest) -> Result<(), String> {
    let content_json = serde_json::to_string(&request).map_err(|_| "unexpected failure during serialize request")?;

    let client = reqwest::Client::new();
    let api_request = client.post(webhook_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(content_json);
    api_request.send().await.map_err(|e| format!("failed to post: {}", e))?;

    Ok(())
}
