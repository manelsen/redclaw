#![allow(dead_code)]
pub mod registry;
pub mod builtin;

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    fn execute(&self, args: Value) -> Result<String>;
}

pub type ToolBox = HashMap<String, Box<dyn Tool>>;
