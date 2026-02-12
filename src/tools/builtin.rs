#![allow(dead_code)]
use anyhow::Result;
use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{Read, BufReader};
use std::path::Path;
use std::process::Command;
use crate::tools::Tool;

pub struct ReadFileTool;
impl Tool for ReadFileTool {
    fn name(&self) -> &str { "read_file" }
    fn description(&self) -> &str { "Read the contents of a file (limit 256KB for safety)" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file to read" }
            },
            "required": ["path"]
        })
    }
    fn execute(&self, args: Value) -> Result<String> {
        let path = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("path is required"))?;
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();
        
        // RAM Safety: Limit read to 256KB to prevent OOM on 2MB hardware
        let limit = 256 * 1024;
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        
        if file_size > limit {
            reader.by_ref().take(limit).read_to_end(&mut buffer)?;
            let mut content = String::from_utf8_lossy(&buffer).to_string();
            content.push_str("\n... (truncated: file exceeds 256KB safety limit)");
            Ok(content)
        } else {
            reader.read_to_end(&mut buffer)?;
            Ok(String::from_utf8_lossy(&buffer).to_string())
        }
    }
}

pub struct WriteFileTool;
impl Tool for WriteFileTool {
    fn name(&self) -> &str { "write_file" }
    fn description(&self) -> &str { "Write content to a file" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file to write" },
                "content": { "type": "string", "description": "Content to write to the file" }
            },
            "required": ["path", "content"]
        })
    }
    fn execute(&self, args: Value) -> Result<String> {
        let path = args["path"].as_str().ok_or_else(|| anyhow::anyhow!("path is required"))?;
        let content = args["content"].as_str().ok_or_else(|| anyhow::anyhow!("content is required"))?;
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok("File written successfully".to_string())
    }
}

pub struct ListDirTool;
impl Tool for ListDirTool {
    fn name(&self) -> &str { "list_dir" }
    fn description(&self) -> &str { "List files and directories in a path" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to list" }
            },
            "required": ["path"]
        })
    }
    fn execute(&self, args: Value) -> Result<String> {
        let path = args["path"].as_str().unwrap_or(".");
        let entries = fs::read_dir(path)?;
        let mut result = String::new();
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                result.push_str("DIR:  ");
            } else {
                result.push_str("FILE: ");
            }
            result.push_str(&entry.file_name().to_string_lossy());
            result.push('\n');
        }
        Ok(result)
    }
}

pub struct ExecTool {
    pub working_dir: String,
}
impl Tool for ExecTool {
    fn name(&self) -> &str { "exec" }
    fn description(&self) -> &str { "Execute a shell command (blocked: rm -rf, format, etc)" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "The shell command to execute" }
            },
            "required": ["command"]
        })
    }
    fn execute(&self, args: Value) -> Result<String> {
        let command = args["command"].as_str().ok_or_else(|| anyhow::anyhow!("command is required"))?;
        
        // Command Guard: Security Blacklist
        let dangerous_patterns = [
            "rm -rf", "rm -r", "mkfs", "format", "dd if=", "> /dev/sd", 
            "shutdown", "reboot", ":(){ :|:& };:", "chmod -R", "chown -R"
        ];
        
        let cmd_lower = command.to_lowercase();
        for pattern in dangerous_patterns {
            if cmd_lower.contains(pattern) {
                return Ok(format!("Security Error: Command blocked. Dangerous pattern '{}' detected.", pattern));
            }
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .output()?;
        
        let mut result = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.stderr.is_empty() {
            result.push_str("\nSTDERR:\n");
            result.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        
        // Output Safety: Limit output size to prevent OOM
        if result.len() > 100 * 1024 {
            result = format!("{}... (truncated: output exceeds 100KB)", &result[..100 * 1024]);
        }
        
        if result.is_empty() {
            result = "(no output)".to_string();
        }
        Ok(result)
    }
}

pub struct WebSearchTool {
    pub api_key: String,
    pub max_results: usize,
}
impl Tool for WebSearchTool {
    fn name(&self) -> &str { "web_search" }
    fn description(&self) -> &str { "Search the web using Brave Search API" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "count": { "type": "integer", "description": "Number of results", "minimum": 1, "maximum": 10 }
            },
            "required": ["query"]
        })
    }
    fn execute(&self, args: Value) -> Result<String> {
        if self.api_key.is_empty() {
            return Ok("Error: Brave API key not configured".to_string());
        }
        let query = args["query"].as_str().ok_or_else(|| anyhow::anyhow!("query is required"))?;
        let count = args["count"].as_u64().unwrap_or(self.max_results as u64) as usize;

        let output = Command::new("curl")
            .arg("-s")
            .arg("-G")
            .arg("--data-urlencode").arg(format!("q={}", query))
            .arg("--data-urlencode").arg(format!("count={}", count))
            .arg("-H").arg(format!("X-Subscription-Token: {}", self.api_key))
            .arg("https://api.search.brave.com/res/v1/web/search")
            .output()?;

        let json: Value = serde_json::from_slice(&output.stdout)?;
        let results = json["web"]["results"].as_array().ok_or_else(|| anyhow::anyhow!("No results found"))?;

        let mut output = format!("Results for: {}\n", query);
        for (i, res) in results.iter().enumerate() {
            output.push_str(&format!("{}. {}\n   {}\n   {}\n", 
                i + 1, 
                res["title"].as_str().unwrap_or(""),
                res["url"].as_str().unwrap_or(""),
                res["description"].as_str().unwrap_or("")
            ));
        }
        Ok(output)
    }
}

pub struct WebFetchTool;
impl Tool for WebFetchTool {
    fn name(&self) -> &str { "web_fetch" }
    fn description(&self) -> &str { "Fetch content from a URL" }
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL to fetch" }
            },
            "required": ["url"]
        })
    }
    fn execute(&self, args: Value) -> Result<String> {
        let url = args["url"].as_str().ok_or_else(|| anyhow::anyhow!("url is required"))?;
        let output = Command::new("curl")
            .arg("-s")
            .arg("-L") // Follow redirects
            .arg(url)
            .output()?;
        
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        
        let text = text.split('<')
            .map(|s| s.split('>').last().unwrap_or(""))
            .collect::<Vec<_>>()
            .join(" ");
        
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let limit = 5000;
        if text.len() > limit {
            Ok(format!("{}... (truncated)", &text[..limit]))
        } else {
            Ok(text)
        }
    }
}

pub struct SysInfoTool;
impl Tool for SysInfoTool {
    fn name(&self) -> &str { "get_sys_info" }
    fn description(&self) -> &str { "Get real-time system and process memory info (RSS)" }
    fn parameters(&self) -> Value { json!({}) }
    fn execute(&self, _args: Value) -> Result<String> {
        let statm = fs::read_to_string("/proc/self/statm")?;
        let parts: Vec<&str> = statm.split_whitespace().collect();
        let pages: u64 = parts[1].parse().unwrap_or(0);
        let rss_kb = pages * 4; // Assuming 4KB pages
        
        let mut output = format!("RedClaw Process Info:\n- Real RAM Usage (RSS): {} KB\n", rss_kb);
        
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmPeak") || line.starts_with("Threads") {
                    output.push_str(&format!("- {}: {}\n", line.split(':').nth(0).unwrap_or(""), line.split(':').nth(1).unwrap_or("").trim()));
                }
            }
        }

        if let Ok(fds) = fs::read_dir("/proc/self/fd") {
            output.push_str(&format!("- Open FDs: {}\n", fds.count()));
        }

        if let Ok(os_info) = fs::read_to_string("/proc/meminfo") {
            let free = os_info.lines().find(|l| l.starts_with("MemAvailable")).unwrap_or("");
            output.push_str(&format!("- System Info: {}\n", free));
        }
        Ok(output)
    }
}
