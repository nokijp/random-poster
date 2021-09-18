extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate tokio;

mod message;
mod random;
mod request;
mod settings;
mod weight;

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
    let settings = read_settings("conf/settings.yaml")?;
    let mut random_picker =
        RandomPicker::from_log_file(
            "conf/message-log.json",
            settings.messages.keys().cloned().collect(),
            settings.environment.weight_type,
            settings.environment.initial_count_type,
        )?;

    let message_id = random_picker.pick();
    let content = SimpleWebhookRequest {
        username: &settings.environment.user_settings.name,
        avatar_url: &settings.environment.user_settings.icon_url,
        message: &settings.messages[message_id],
    };
    post(&settings.environment.webhook_url, &content).await?;

    random_picker.write_log()?;

    Ok(())
}
