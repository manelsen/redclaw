#![allow(dead_code)]
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use time::{OffsetDateTime, format_description};

pub struct MemoryStore {
    memory_dir: PathBuf,
    memory_file: PathBuf,
}

impl MemoryStore {
    pub fn new(workspace: &Path) -> Self {
        let memory_dir = workspace.join("memory");
        let memory_file = memory_dir.join("MEMORY.md");

        if !memory_dir.exists() {
            fs::create_dir_all(&memory_dir).ok();
        }

        Self {
            memory_dir: memory_dir.clone(),
            memory_file,
        }
    }

    pub fn workspace(&self) -> PathBuf {
        self.memory_dir.parent().unwrap_or(&self.memory_dir).to_path_buf()
    }

    fn get_today_file(&self) -> PathBuf {
        let now = OffsetDateTime::now_utc();
        let year = now.year();
        let month = u8::from(now.month());
        let day = now.day();
        let today = format!("{:04}{:02}{:02}", year, month, day);
        let month_dir = format!("{:04}{:02}", year, month);
        self.memory_dir.join(month_dir).join(format!("{}.md", today))
    }

    pub fn read_long_term(&self) -> String {
        fs::read_to_string(&self.memory_file).unwrap_or_default()
    }

    pub fn write_long_term(&self, content: &str) -> Result<()> {
        fs::write(&self.memory_file, content)?;
        Ok(())
    }

    pub fn read_today(&self) -> String {
        fs::read_to_string(self.get_today_file()).unwrap_or_default()
    }

    pub fn append_today(&self, content: &str) -> Result<()> {
        let today_file = self.get_today_file();
        if let Some(parent) = today_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let existing = fs::read_to_string(&today_file).unwrap_or_default();
        let now = OffsetDateTime::now_utc();
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        let date_str = now.format(&format).unwrap();

        let new_content = if existing.is_empty() {
            format!("# {}\n\n{}", date_str, content)
        } else {
            format!("{}\n{}", existing, content)
        };

        fs::write(today_file, new_content)?;
        Ok(())
    }

    pub fn get_memory_context(&self) -> String {
        let mut parts = Vec::new();

        let long_term = self.read_long_term();
        if !long_term.is_empty() {
            parts.push(format!("## Long-term Memory\n\n{}", long_term));
        }

        let mut recent_notes = Vec::new();
        let now = OffsetDateTime::now_utc();
        
        for i in 0..3 {
            let date = now - time::Duration::days(i);
            let year = date.year();
            let month = u8::from(date.month());
            let day = date.day();
            let date_str = format!("{:04}{:02}{:02}", year, month, day);
            let month_dir = format!("{:04}{:02}", year, month);
            let file_path = self.memory_dir.join(month_dir).join(format!("{}.md", date_str));

            if let Ok(data) = fs::read_to_string(file_path) {
                recent_notes.push(data);
            }
        }

        if !recent_notes.is_empty() {
            parts.push(format!("## Recent Daily Notes\n\n{}", recent_notes.join("\n\n---\n\n")));
        }

        if parts.is_empty() {
            return String::new();
        }

        format!("# Memory\n\n{}", parts.join("\n\n---\n\n"))
    }
}
