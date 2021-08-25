use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(PartialEq, Deserialize, Debug)]
pub struct Settings {
    pub environment: EnvironmentSettings,
    pub messages: Vec<String>,
}

#[derive(PartialEq, Deserialize, Debug)]
pub struct EnvironmentSettings {
    pub webhook_url: String,
    pub weight_bias: f64,
    #[serde(rename = "user", default = "empty_user_settings")]
    pub user_settings: UserSettings,
}

#[derive(PartialEq, Eq, Deserialize, Debug)]
pub struct UserSettings {
    pub name: Option<String>,
    pub icon_url: Option<String>,
}

fn empty_user_settings() -> UserSettings {
    UserSettings { name: None, icon_url: None }
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
    fn read_settings_can_read_a_yaml_file_which_contains_user_settings() {
        let input = indoc! {r#"
            environment:
                webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
                weight_bias: 2.0
                user:
                    name: "user_name"
                    icon_url: "https://example.com/icon.png"
            messages:
                - "abc"
                - "def"
        "#};
        let expected = Settings {
            environment: EnvironmentSettings {
                webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY".to_string(),
                weight_bias: 2.0,
                user_settings: UserSettings {
                    name: Some("user_name".to_string()),
                    icon_url: Some("https://example.com/icon.png".to_string()),
                },
            },
            messages: vec![
                "abc".to_string(),
                "def".to_string(),
            ],
        };

        assert_eq!(Ok(expected), from_str(input));
    }

    #[test]
    fn read_settings_can_read_a_yaml_file_which_does_not_contain_user_settings() {
        let input = indoc! {r#"
            environment:
                webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY"
                weight_bias: 2.0
            messages:
                - "abc"
                - "def"
        "#};
        let expected = Settings {
            environment: EnvironmentSettings {
                webhook_url: "https://discord.com/api/webhooks/XXXX/YYYY".to_string(),
                weight_bias: 2.0,
                user_settings: UserSettings {
                    name: None,
                    icon_url: None,
                },
            },
            messages: vec![
                "abc".to_string(),
                "def".to_string(),
            ],
        };

        assert_eq!(Ok(expected), from_str(input));
    }

    fn from_str(input: &str) -> Result<Settings, String> {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", input).unwrap();
        read_settings(file.path())
    }
}
