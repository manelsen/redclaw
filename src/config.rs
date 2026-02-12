#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub agents: AgentsConfig,
    pub providers: ProvidersConfig,
    pub tools: ToolsConfig,
    #[serde(default)]
    pub channels: ChannelsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ChannelsConfig {
    pub telegram: Option<TelegramConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub token: String,
    pub allow_from: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentsConfig {
    pub defaults: AgentDefaults,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentDefaults {
    pub workspace: String,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub max_tool_iterations: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProvidersConfig {
    pub openai: Option<ProviderConfig>,
    pub gemini: Option<ProviderConfig>,
    pub openrouter: Option<ProviderConfig>,
    pub zhipu: Option<ProviderConfig>,
    pub vllm: Option<ProviderConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub api_key: String,
    pub api_base: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolsConfig {
    pub web: WebToolsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebToolsConfig {
    pub search: WebSearchConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSearchConfig {
    pub api_key: String,
    pub max_results: usize,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn workspace_path(&self) -> PathBuf {
        let path = &self.agents.defaults.workspace;
        if path.starts_with('~') {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(&path[2..]);
            }
        }
        PathBuf::from(path)
    }
}
