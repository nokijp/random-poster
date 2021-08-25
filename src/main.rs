extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate tokio;

mod settings;
mod random;
mod request;

use settings::read_settings;
use random::RandomPicker;
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
    let mut random_picker = RandomPicker::from_log_file("message-log.json", settings.messages, settings.environment.weight_bias)?;

    let message = random_picker.pick().clone();
    let content = SimpleWebhookRequest {
        username: settings.environment.user_settings.name,
        avatar_url: settings.environment.user_settings.icon_url,
        content: message,
    };
    post(&settings.environment.webhook_url, &content).await?;

    random_picker.write_log()?;

    Ok(())
}
