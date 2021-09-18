use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::random::InitialCountType;
use super::weight::WeightType;
use super::message::Message;

#[derive(PartialEq, Deserialize, Debug)]
pub struct Settings {
    pub environment: EnvironmentSettings,
    pub messages: HashMap<String, Message>,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct EnvironmentSettings {
    pub webhook_url: String,
    pub weight_type: WeightType,
    #[serde(default = "InitialCountType::default")]
    pub initial_count_type: InitialCountType,
    #[serde(rename = "user", default = "UserSettings::default")]
    pub user_settings: UserSettings,
}

#[derive(PartialEq, Eq, Deserialize, Debug)]
pub struct UserSettings {
    pub name: Option<String>,
    pub icon_url: Option<String>,
}

impl UserSettings {
    fn default() -> UserSettings {
        UserSettings { name: None, icon_url: None }
    }
}

pub fn read_settings<P: AsRef<Path>>(path: P) -> Result<Settings, String> {
    let path_ref = path.as_ref();
    let mut file = File::open(path_ref).map_err(|_| format!("could not open file: {}", path_ref.display()))?;
    let mut file_reader = BufReader::new(&mut file);

    let settings = serde_yaml::from_reader(&mut file_reader).map_err(|e| format!("failed to read settings: {}", e))?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    extern crate indoc;
    extern crate tempfile;

    use super::*;
    use indoc::indoc;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn read_settings_can_read_a_yaml_file_which_contains_all_settings() {
        let input = indoc! {r#"
            environment:
              webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
              weight_type:
                type: "Uniform"
              initial_count_type: "Min"
              user:
                name: "user_name"
                icon_url: "https://example.com/icon.png"
            messages:
              abc: "message1"
              def: "message2"
        "#};
        let expected = Settings {
            environment: EnvironmentSettings {
                webhook_url: String::from("https://discord.com/api/webhooks/XXXX/YYYY"),
                weight_type: WeightType::Uniform,
                initial_count_type: InitialCountType::Min,
                user_settings: UserSettings {
                    name: Some(String::from("user_name")),
                    icon_url: Some(String::from("https://example.com/icon.png")),
                },
            },
            messages: vec![
                (String::from("abc"), Message::String(String::from("message1"))),
                (String::from("def"), Message::String(String::from("message2"))),
            ].into_iter().collect(),
        };

        assert_eq!(Ok(expected), from_str(input));
    }

    #[test]
    fn read_settings_can_read_a_yaml_file_which_does_not_contain_optional_settings() {
        let input = indoc! {r#"
            environment:
              webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
              weight_type:
                type: "Uniform"
            messages:
              abc: "message1"
              def: "message2"
        "#};
        let expected = Settings {
            environment: EnvironmentSettings {
                webhook_url: String::from("https://discord.com/api/webhooks/XXXX/YYYY"),
                weight_type: WeightType::Uniform,
                initial_count_type: InitialCountType::Zero,
                user_settings: UserSettings {
                    name: None,
                    icon_url: None,
                },
            },
            messages: vec![
                (String::from("abc"), Message::String(String::from("message1"))),
                (String::from("def"), Message::String(String::from("message2"))),
            ].into_iter().collect(),
        };

        assert_eq!(Ok(expected), from_str(input));
    }

    #[test]
    fn read_settings_can_read_min_only_weight() {
        let input = indoc! {r#"
            environment:
              webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
              weight_type:
                type: "MinOnly"
            messages:
              abc: "message1"
              def: "message2"
        "#};
        let expected = Settings {
            environment: EnvironmentSettings {
                webhook_url: String::from("https://discord.com/api/webhooks/XXXX/YYYY"),
                weight_type: WeightType::MinOnly,
                initial_count_type: InitialCountType::Zero,
                user_settings: UserSettings {
                    name: None,
                    icon_url: None,
                },
            },
            messages: vec![
                (String::from("abc"), Message::String(String::from("message1"))),
                (String::from("def"), Message::String(String::from("message2"))),
            ].into_iter().collect(),
        };

        assert_eq!(Ok(expected), from_str(input));
    }

    #[test]
    fn read_settings_can_read_linear_weight() {
        let input = indoc! {r#"
            environment:
              webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
              weight_type:
                type: "Linear"
                bias: 10.0
            messages:
              abc: "message1"
              def: "message2"
        "#};
        let expected = Settings {
            environment: EnvironmentSettings {
                webhook_url: String::from("https://discord.com/api/webhooks/XXXX/YYYY"),
                weight_type: WeightType::Linear { bias: 10.0 },
                initial_count_type: InitialCountType::Zero,
                user_settings: UserSettings {
                    name: None,
                    icon_url: None,
                },
            },
            messages: vec![
                (String::from("abc"), Message::String(String::from("message1"))),
                (String::from("def"), Message::String(String::from("message2"))),
            ].into_iter().collect(),
        };

        assert_eq!(Ok(expected), from_str(input));
    }

    #[test]
    fn read_settings_can_read_boltzmann_weight() {
        let input = indoc! {r#"
            environment:
              webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
              weight_type:
                type: "Boltzmann"
                beta: 10.0
            messages:
              abc: "message1"
              def: "message2"
        "#};
        let expected = Settings {
            environment: EnvironmentSettings {
                webhook_url: String::from("https://discord.com/api/webhooks/XXXX/YYYY"),
                weight_type: WeightType::Boltzmann { beta: 10.0 },
                initial_count_type: InitialCountType::Zero,
                user_settings: UserSettings {
                    name: None,
                    icon_url: None,
                },
            },
            messages: vec![
                (String::from("abc"), Message::String(String::from("message1"))),
                (String::from("def"), Message::String(String::from("message2"))),
            ].into_iter().collect(),
        };

        assert_eq!(Ok(expected), from_str(input));
    }

    #[test]
    fn read_settings_can_read_mixed_messages() {
        let input = indoc! {r#"
            environment:
              webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
              weight_type:
                type: "Uniform"
            messages:
              abc: "message1"
              def:
                content: "message2"
                embeds:
                  - title: "title1"
                    url: "https://example.com/1"
                    thumbnail:
                      url: "https://example.com/thumbnail1.png"
                  - title: "title2"
                    url: "https://example.com/2"
                    thumbnail:
                      url: "https://example.com/thumbnail2.png"
        "#};
        let expected = Settings {
            environment: EnvironmentSettings {
                webhook_url: String::from("https://discord.com/api/webhooks/XXXX/YYYY"),
                weight_type: WeightType::Uniform,
                initial_count_type: InitialCountType::Zero,
                user_settings: UserSettings {
                    name: None,
                    icon_url: None,
                },
            },
            messages: vec![
                (String::from("abc"), Message::String(String::from("message1"))),
                (String::from("def"), Message::WithEmbeds {
                    content: Some(String::from("message2")),
                    embeds: vec![
                        serde_json::json!({
                            "title": "title1",
                            "url": "https://example.com/1",
                            "thumbnail": {
                                "url": "https://example.com/thumbnail1.png",
                            },
                        }),
                        serde_json::json!({
                            "title": "title2",
                            "url": "https://example.com/2",
                            "thumbnail": {
                                "url": "https://example.com/thumbnail2.png",
                            },
                        }),
                    ],
                }),
            ].into_iter().collect(),
        };

        assert_eq!(Ok(expected), from_str(input));
    }

    fn from_str(input: &str) -> Result<Settings, String> {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", input).unwrap();
        read_settings(file.path())
    }
}
