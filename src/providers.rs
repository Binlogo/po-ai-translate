use std::collections::HashMap;

use anyhow::Result;

pub mod moonshot;

pub trait TranslationProvider {
    async fn translate(&self, msgids: &[&str], lang: &str) -> Result<HashMap<String, String>>;
}

pub async fn moonshot_translate(msgids: &[&str], lang: &str) -> Result<HashMap<String, String>> {
    let api_key = std::env::var("MOONSHOT_API_KEY").expect("MOONSHOT_API_KEY must be set");
    let provider = moonshot::MoonshotProvider::new(api_key);
    provider.translate(msgids, lang).await
}