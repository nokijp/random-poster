use serde::{Serialize, Serializer, Deserialize, ser::SerializeStruct};

#[derive(PartialEq, Deserialize, Debug)]
#[serde(untagged)]
pub enum Message {
    String(String),
    WithEmbeds {
        content: Option<String>,
        embeds: Vec<serde_json::Value>,
    },
}

impl Serialize for Message {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Message::String(content) => {
                let mut s = serializer.serialize_struct("Message", 1)?;
                s.serialize_field("content", &content)?;
                s.end()
            },
            Message::WithEmbeds { content: None, embeds } => {
                let mut s = serializer.serialize_struct("Message", 1)?;
                s.serialize_field("embeds", &embeds)?;
                s.end()
            },
            Message::WithEmbeds { content, embeds } => {
                let mut s = serializer.serialize_struct("Message", 2)?;
                s.serialize_field("content", &content)?;
                s.serialize_field("embeds", &embeds)?;
                s.end()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate indoc;

    use super::*;
    use indoc::indoc;
    use serde_json::Value;

    #[test]
    fn read_settings_can_serialize_string() {
        let message = Message::String(String::from("message"));
        let expected = indoc! {r#"
            {
                "content": "message"
            }
        "#};

        let json = serde_json::to_string(&message).unwrap();
        assert_eq!(to_json_value(&expected), to_json_value(&json));
    }

    #[test]
    fn read_settings_can_serialize_with_embeds() {
        let message = Message::WithEmbeds {
            content: Some(String::from("message")),
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
        };
        let expected = indoc! {r#"
            {
                "content": "message",
                "embeds": [
                    {
                        "title": "title1",
                        "url": "https://example.com/1",
                        "thumbnail": {
                            "url": "https://example.com/thumbnail1.png"
                        }
                    },
                    {
                        "title": "title2",
                        "url": "https://example.com/2",
                        "thumbnail": {
                            "url": "https://example.com/thumbnail2.png"
                        }
                    }
                ]
            }
        "#};

        let json = serde_json::to_string(&message).unwrap();
        assert_eq!(to_json_value(&expected), to_json_value(&json));
    }

    #[test]
    fn read_settings_can_serialize_with_embeds_without_content() {
        let message = Message::WithEmbeds {
            content: None,
            embeds: vec![
                serde_json::json!({
                    "title": "title1",
                    "url": "https://example.com/1",
                    "thumbnail": {
                        "url": "https://example.com/thumbnail1.png",
                    },
                }),
            ],
        };
        let expected = indoc! {r#"
            {
                "embeds": [
                    {
                        "title": "title1",
                        "url": "https://example.com/1",
                        "thumbnail": {
                            "url": "https://example.com/thumbnail1.png"
                        }
                    }
                ]
            }
        "#};

        let json = serde_json::to_string(&message).unwrap();
        assert_eq!(to_json_value(&expected), to_json_value(&json));
    }

    fn to_json_value(s: &str) -> Value {
        serde_json::from_str(s).unwrap()
    }
}
