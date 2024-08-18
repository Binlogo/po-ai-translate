use std::collections::HashMap;

use crate::providers::TranslationProvider;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub struct MoonshotProvider {
    api_key: String,
    client: Client,
}

impl MoonshotProvider {
    pub fn new(api_key: String) -> Self {
        MoonshotProvider {
            api_key,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct MoonshotResponse {
    choices: Vec<Choice>,
}

const SYSTEM_PROMPT: &str = r#"
You are a professional translator that translates English to specific language.
Content is from gettext po file, I'll provide msgids and context, you job is to translate it to given language.
- Keep escape words, such as: `\"`, `\n`, even after json serialize and deserialize, 

Input is in JSON format, with following fields:
- msgids: array of msgids
- context(optional): context of the msgids
- lang: language to translate to

Output is in JSON format, with following fields:
- translations: array of translations, exactly same order and count of msgids.

其他要求：
- 最终结果必须保留转义符，例如：`\"`, `\n`等, 避免在 JSON 转换中丢失。
"#;

impl TranslationProvider for MoonshotProvider {
    async fn translate(&self, msgids: &[&str], lang: &str) -> Result<HashMap<String, String>> {
        let url = "https://api.moonshot.cn/v1/chat/completions";

        let message = json!({
            "role": "user",
            "content": json!({
                "msgids": msgids,
                "lang": lang
            }).to_string()
        });

        let body = json!({
            "model": "moonshot-v1-8k",
            "messages": [
                {
                    "role": "system",
                    "content": SYSTEM_PROMPT,
                },
                message
            ],
            "temperature": 0.3
        });

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .bearer_auth(self.api_key.clone())
            .json(&body)
            .send()
            .await?;

        // Print status code and headers for debugging
        println!("Status: {}", response.status());
        println!("Headers: {:#?}", response.headers());

        let bytes = response.bytes().await?;

        // Print raw response body for debugging
        // println!("Raw response: {}", String::from_utf8_lossy(&bytes));

        let response: MoonshotResponse = serde_json::from_slice(&bytes)?;

        let content = response
            .choices
            .first()
            .ok_or_else(|| anyhow::anyhow!("No choices in response"))?
            .message
            .content
            .clone();
        let parsed: serde_json::Value = serde_json::from_str(&content)?;

        let translations = parsed["translations"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Translations not found or not an array"))?
            .iter().zip(msgids)
            .map(|(v, msgid)| (msgid.to_string(), v.as_str().unwrap_or_default().to_string()))
            .collect();

        Ok(translations)
    }
}
