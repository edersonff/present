use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AskRequest {
    #[serde(deserialize_with = "require_non_empty")]
    pub message: String,
    #[serde(deserialize_with = "require_two_or_more")]
    pub options: Vec<String>,
    #[serde(default)]
    pub multiple: bool,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum AskResponse {
    Selected { selected: Vec<String> },
    Cancelled { cancelled: bool },
}

impl AskResponse {
    pub fn selected(items: Vec<String>) -> Self {
        AskResponse::Selected { selected: items }
    }
    pub fn cancelled() -> Self {
        AskResponse::Cancelled { cancelled: true }
    }
}

#[derive(Debug, Deserialize)]
pub struct ProgressUpdate {
    pub current: u64,
    pub total: u64,
    #[serde(default)]
    pub label: String,
}

fn require_non_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    if raw.trim().is_empty() {
        return Err(serde::de::Error::custom(
            "message is empty, present needs something to ask",
        ));
    }
    Ok(raw)
}

fn require_two_or_more<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = Vec::<String>::deserialize(deserializer)?;
    match raw.len() {
        0 => Err(serde::de::Error::custom(
            "options is empty, present needs at least two to ask",
        )),
        1 => Err(serde::de::Error::custom(
            "only one option was given, nothing to ask. pass it through or add another",
        )),
        _ => Ok(raw),
    }
}

impl AskRequest {
    pub fn from_json_str(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

impl ProgressUpdate {
    pub fn from_json_str(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

pub fn write_json<T: Serialize>(value: &T) {
    let mut out = std::io::stdout();
    let _ = serde_json::to_writer(&mut out, value);
    let _ = out.write_all(b"\n");
}

pub fn read_stdin_to_string() -> std::io::Result<String> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
