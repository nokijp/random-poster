extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate tokio;

mod settings;
mod request;

use settings::read_settings;
use request::{SimpleWebhookRequest, post};

#[tokio::main]
async fn main() {
    let result = run().await;
    if let Err(message) = result {
        eprintln!("{}", message);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let settings = read_settings("settings.yaml")?;

    let content = SimpleWebhookRequest {
        username: settings.environment.user_settings.name,
        avatar_url: settings.environment.user_settings.icon_url,
        content: "message".to_string(),
    };
    post(&settings.environment.webhook_url, &content).await?;

    Ok(())
}
