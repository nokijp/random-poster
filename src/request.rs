use serde::Serialize;

use super::message::Message;

#[derive(Serialize)]
pub struct SimpleWebhookRequest<'a> {
    pub username: &'a Option<String>,
    pub avatar_url: &'a Option<String>,
    #[serde(flatten)]
    pub message: &'a Message,
}

pub async fn post(webhook_url: &str, request: &SimpleWebhookRequest<'_>) -> Result<(), String> {
    let content_json = serde_json::to_string(request).unwrap();

    let client = reqwest::Client::new();
    let api_request = client.post(webhook_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(content_json);
    api_request.send().await.map_err(|e| format!("failed to post: {}", e))?;

    Ok(())
}
