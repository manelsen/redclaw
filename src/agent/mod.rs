#![allow(dead_code)]
pub mod llm;
pub mod memory;
pub mod channels;

use anyhow::Result;
use crate::config::Config;
use crate::tools::registry::ToolRegistry;
use crate::agent::llm::{LLMClient, Message};
use crate::agent::memory::MemoryStore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub messages: Vec<Message>,
}

pub struct Agent {
    client: LLMClient,
    memory: MemoryStore,
    tools: ToolRegistry,
    max_iterations: usize,
    session_key: String,
}

impl Agent {
    pub fn new(config: &Config, client: LLMClient, tools: ToolRegistry) -> Self {
        let workspace = config.workspace_path();
        let memory = MemoryStore::new(&workspace);
        Self {
            client,
            memory,
            tools,
            max_iterations: config.agents.defaults.max_tool_iterations,
            session_key: "default".to_string(),
        }
    }

    pub fn set_session(&mut self, key: &str) {
        self.session_key = key.to_string();
    }

    fn get_session_path(&self) -> PathBuf {
        self.memory.workspace().join("sessions").join(format!("{}.json", self.session_key))
    }

    fn load_session(&self) -> Session {
        let path = self.get_session_path();
        if let Ok(file) = fs::File::open(path) {
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or(Session { messages: Vec::new() })
        } else {
            Session { messages: Vec::new() }
        }
    }

    fn save_session(&self, session: &Session) -> Result<()> {
        let path = self.get_session_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let file = fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, session)?;
        Ok(())
    }

    pub fn run(&mut self, user_input: &str) -> Result<String> {
        let mut session = self.load_session();
        
        let mut bootstrap_context = String::new();
        let bootstrap_files = ["USER.md", "SOUL.md", "IDENTITY.md"];
        let workspace = self.memory.workspace();
        for file in bootstrap_files {
            let path = workspace.join(file);
            if let Ok(content) = fs::read_to_string(path) {
                bootstrap_context.push_str(&format!("## {}\n\n{}\n\n", file, content));
            }
        }

        let system_prompt = format!(
            "You are RedClaw, an ultra-efficient embedded AI agent. Keep responses brief. \
             If you have enough information from tool results, provide a final answer immediately. \
             Avoid repeating the same tool calls with identical parameters.\n\n{}\n\n{}",
            bootstrap_context,
            self.memory.get_memory_context()
        );

        let mut api_messages = Vec::new();
        api_messages.push(Message {
            role: "system".to_string(),
            content: Some(system_prompt),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        });

        let history_limit = 10;
        let mut start = if session.messages.len() > history_limit {
            session.messages.len() - history_limit
        } else {
            0
        };
        
        while start < session.messages.len() && session.messages[start].role == "tool" {
            start += 1;
        }

        for msg in &session.messages[start..] {
            api_messages.push(msg.clone());
        }

        let current_user_msg = Message {
            role: "user".to_string(),
            content: Some(user_input.to_string()),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        };
        api_messages.push(current_user_msg.clone());
        session.messages.push(current_user_msg);

        let mut iteration = 0;
        let mut final_content = String::new();

        while iteration < self.max_iterations {
            iteration += 1;
            let tool_defs = self.tools.get_definitions();
            let response = self.client.chat(&api_messages, Some(&tool_defs))?;

            api_messages.push(response.clone());
            session.messages.push(response.clone());

            if let Some(tool_calls) = &response.tool_calls {
                if let Some(calls) = tool_calls.as_array() {
                    for tc in calls {
                        let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let function = tc.get("function").ok_or_else(|| anyhow::anyhow!("No function in tool call"))?;
                        let name = function.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let args_str = function.get("arguments").and_then(|v| v.as_str()).unwrap_or("{}");
                        
                        println!("  Action: {}({})", name, args_str);

                        let args: Value = serde_json::from_str(args_str)?;
                        let result = match self.tools.execute(name, args) {
                            Ok(res) => res,
                            Err(e) => format!("Error: {}", e),
                        };

                        let tool_msg = Message {
                            role: "tool".to_string(),
                            content: Some(result),
                            name: Some(name.to_string()),
                            tool_call_id: Some(id.to_string()),
                            tool_calls: None,
                        };
                        api_messages.push(tool_msg.clone());
                        session.messages.push(tool_msg);
                    }
                }
            } else {
                final_content = response.content.clone().unwrap_or_default();
                break;
            }
        }

        // If we hit the limit without a final answer, force one last completion without tools
        if final_content.is_empty() && iteration >= self.max_iterations {
            if let Ok(last_res) = self.client.chat(&api_messages, None) {
                final_content = last_res.content.unwrap_or_default();
                session.messages.push(Message {
                    role: "assistant".to_string(),
                    content: Some(final_content.clone()),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                });
            }
        }

        if !final_content.is_empty() {
            let _ = self.memory.append_today(&format!("User: {}\nAssistant: {}\n", user_input, final_content));
        }

        self.save_session(&session)?;
        api_messages.clear();
        api_messages.shrink_to_fit();
        
        Ok(final_content)
    }

    pub fn summarize(&self, messages: &[Message]) -> Result<String> {
        let prompt = "Provide a very concise summary of this conversation segment, preserving core context and key points.\n\nCONVERSATION:\n";
        let summary_messages = vec![Message {
            role: "user".to_string(),
            content: Some(format!("{}{}", prompt, self.format_messages(messages))),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        }];

        let response = self.client.chat(&summary_messages, None)?;
        Ok(response.content.unwrap_or_default())
    }

    fn format_messages(&self, messages: &[Message]) -> String {
        messages.iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .map(|m| {
                let content = m.content.as_deref().unwrap_or("");
                format!("{}: {}", m.role, content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
