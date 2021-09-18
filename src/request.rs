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
    let response = api_request.send().await.map_err(|e| format!("failed to post: {}", e))?;

    let response_status = response.status();
    if !response_status.is_success() {
        return if let Ok(response_body) = response.text().await {
            Err(format!("failed with {}: {}", response_status, response_body))
        } else {
            Err(format!("failed with {}", response_status))
        }
    }

    Ok(())
}
