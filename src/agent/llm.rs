use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use crate::config::ProviderConfig;
use serde_json::Value;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDefinition {
    pub r#type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

pub struct LLMClient {
    pub api_key: String,
    pub api_base: String,
    pub model: String,
}

impl LLMClient {
    pub fn new(config: &ProviderConfig, default_base: &str, model: &str) -> Self {
        Self {
            api_key: config.api_key.clone(),
            api_base: config.api_base.clone().unwrap_or_else(|| default_base.to_string()),
            model: model.to_string(),
        }
    }

    pub fn chat(&self, messages: &[Message], tools: Option<&[ToolDefinition]>) -> Result<Message> {
        // Gemini/OpenRouter compatibility: ensure no null content
        let sanitized_messages: Vec<Message> = messages.iter().map(|m| {
            let mut new_m = m.clone();
            if new_m.content.is_none() && new_m.tool_calls.is_none() {
                new_m.content = Some("".to_string());
            }
            new_m
        }).collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": sanitized_messages,
        });

        if let Some(t) = tools {
            if !t.is_empty() {
                if let Some(obj) = body.as_object_mut() {
                    obj.insert("tools".to_string(), serde_json::json!(t));
                }
            }
        }

        // Use CURL with STDIN for safety and large payloads
        use std::process::Stdio;
        let mut child = Command::new("curl")
            .arg("-s")
            .arg("-X").arg("POST")
            .arg("--connect-timeout").arg("15")
            .arg("--max-time").arg("120")
            .arg(&format!("{}/chat/completions", self.api_base))
            .arg("-H").arg(&format!("Authorization: Bearer {}", self.api_key))
            .arg("-H").arg("Content-Type: application/json")
            .arg("-H").arg("HTTP-Referer: https://github.com/redclaw") // Required by OpenRouter
            .arg("-d").arg("@-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        {
            let stdin = child.stdin.as_mut().ok_or_else(|| anyhow!("Failed to open stdin"))?;
            serde_json::to_writer(stdin, &body)?;
        }

        let output = child.wait_with_output()?;
        
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("CURL Network Error: {}", err));
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        
        // Check for API errors before parsing as success
        let val: Value = serde_json::from_str(&stdout_str)
            .map_err(|e| anyhow!("Failed to parse JSON response: {}. Body: {}", e, stdout_str))?;
        
        if let Some(error) = val.get("error") {
            let msg = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown API Error");
            let code = error.get("code").map(|c| c.to_string()).unwrap_or_else(|| "no code".to_string());
            return Err(anyhow!("LLM Provider Error ({}): {}", code, msg));
        }

        // Handle potential non-JSON or error JSON responses
        let chat_resp: ChatResponse = serde_json::from_value(val)
            .map_err(|e| anyhow!("Failed to map LLM response to ChatResponse: {}. Body: {}", e, stdout_str))?;
        
        chat_resp.choices.into_iter().next()
            .map(|c| c.message)
            .ok_or_else(|| anyhow!("No choices in LLM response: {}", stdout_str))
    }
}
