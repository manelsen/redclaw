#![allow(dead_code)]
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json::Value;
use crate::tools::{Tool, ToolBox};
use crate::agent::llm::ToolDefinition;

pub struct ToolRegistry {
    tools: ToolBox,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| {
            ToolDefinition {
                r#type: "function".to_string(),
                function: crate::agent::llm::FunctionDefinition {
                    name: t.name().to_string(),
                    description: t.description().to_string(),
                    parameters: t.parameters(),
                },
            }
        }).collect()
    }

    pub fn execute(&self, name: &str, args: Value) -> Result<String> {
        let tool = self.tools.get(name).ok_or_else(|| anyhow!("Tool {} not found", name))?;
        tool.execute(args)
    }
}
