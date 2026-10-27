use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::http::Params as HttpParams;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rules {
    pub interval: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Params {
    Http(HttpParams),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: String,
    pub task: Rules,
    pub params: Params,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub notify: Option<Notify>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Notify {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub send: SendMode,
    #[serde(default)]
    pub format: NotifyFormat,
    #[serde(default)]
    pub template: Option<String>,
}

fn default_method() -> String {
    "POST".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize)]
pub enum SendMode {
    #[default]
    #[serde(rename = "all")]
    All,
    #[serde(rename = "non-200")]
    Non200,
    #[serde(rename = "errors")]
    Errors,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize, Serialize)]
pub enum NotifyFormat {
    #[default]
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "plaintext")]
    Plaintext,
}