#![allow(dead_code)]
use anyhow::Result;
use serde::Deserialize;
use crate::agent::Agent;
use std::time::Duration;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
struct TgUpdate {
    update_id: i64,
    message: Option<TgMessage>,
}

#[derive(Deserialize)]
struct TgMessage {
    chat: TgChat,
    text: Option<String>,
    from: Option<TgUser>,
}

#[derive(Deserialize)]
struct TgChat {
    id: i64,
}

#[derive(Deserialize)]
struct TgUser {
    id: i64,
    username: Option<String>,
}

#[derive(Deserialize)]
struct TgResponse<T> {
    ok: bool,
    result: T,
}

pub struct TelegramBot {
    token: String,
    allowed_users: Vec<String>,
}

impl TelegramBot {
    pub fn new(token: String, allowed_users: Vec<String>) -> Self {
        Self { token, allowed_users }
    }

    pub fn run(&self, agent: &mut Agent) -> Result<()> {
        let mut offset = 0;
        println!("Telegram Bot started (Resilient Pipe Mode).");

        loop {
            let url = format!("https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=30", self.token, offset);
            
            let child = Command::new("curl")
                .arg("-s")
                .arg("--connect-timeout").arg("10")
                .arg("--max-time").arg("45")
                .arg(url)
                .stdout(Stdio::piped())
                .spawn();

            match child {
                Ok(mut c) => {
                    let stdout = c.stdout.take().unwrap();
                    if let Ok(tg_res) = serde_json::from_reader::<_, TgResponse<Vec<TgUpdate>>>(stdout) {
                        for update in tg_res.result {
                            offset = update.update_id + 1;
                            if let Some(msg) = update.message {
                                if let Err(e) = self.handle_message(agent, msg) {
                                    eprintln!("Error handling message: {}", e);
                                }
                            }
                        }
                    }
                    let _ = c.wait();
                }
                _ => {
                    std::thread::sleep(Duration::from_secs(5));
                }
            }
        }
    }

    fn handle_message(&self, agent: &mut Agent, msg: TgMessage) -> Result<()> {
        let chat_id = msg.chat.id;
        let text = msg.text.unwrap_or_default();
        let user_id = msg.from.as_ref().map(|u| u.id.to_string()).unwrap_or_default();
        let username = msg.from.as_ref().and_then(|u| u.username.clone()).unwrap_or_else(|| "unknown".to_string());

        if !self.allowed_users.is_empty() && !self.allowed_users.contains(&user_id) && !self.allowed_users.contains(&username) {
            println!("Unauthorized user: {}", username);
            return Ok(());
        }

        crate::utils::print_box_top(&format!("User ({})", username));
        crate::utils::print_box_line(&text);
        println!("");
        
        // Set session key to chat_id for persistence
        agent.set_session(&chat_id.to_string());

        // Typing indicator
        let _ = Command::new("curl")
            .arg("-s").arg("-X").arg("POST")
            .arg("--connect-timeout").arg("5")
            .arg("--max-time").arg("10")
            .arg(format!("https://api.telegram.org/bot{}/sendChatAction", self.token))
            .arg("-H").arg("Content-Type: application/json")
            .arg("-d").arg(serde_json::json!({"chat_id": chat_id, "action": "typing"}).to_string())
            .output();

        match agent.run(&text) {
            Ok(response) => {
                let response = if response.is_empty() { "I processed your request but have no text response.".to_string() } else { response };
                println!("  Claw:");
                crate::utils::print_box_line(&response);
                crate::utils::print_box_bottom();
                self.send_message(chat_id, &response)?;
            }
            Err(e) => {
                println!("  Error:");
                crate::utils::print_box_line(&format!("{}", e));
                crate::utils::print_box_bottom();
                self.send_message(chat_id, &format!("Agent Error: {}", e))?;
            }
        }
        Ok(())
    }

    fn send_message(&self, chat_id: i64, text: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        
        let send = |mode: Option<&str>| -> Result<bool> {
            let mut payload = serde_json::json!({
                "chat_id": chat_id,
                "text": text
            });
            if let Some(m) = mode {
                payload.as_object_mut().unwrap().insert("parse_mode".to_string(), serde_json::json!(m));
            }

            let mut child = Command::new("curl")
                .arg("-s")
                .arg("-X").arg("POST")
                .arg("--connect-timeout").arg("10")
                .arg("--max-time").arg("30")
                .arg(&url)
                .arg("-H").arg("Content-Type: application/json")
                .arg("-d").arg("@-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            if let Some(mut stdin) = child.stdin.take() {
                serde_json::to_writer(&mut stdin, &payload)?;
            }

            let output = child.wait_with_output()?;
            if !output.status.success() || !String::from_utf8_lossy(&output.stdout).contains("\"ok\":true") {
                if mode.is_none() {
                    eprintln!("Telegram Final Failure: {}", String::from_utf8_lossy(&output.stdout));
                }
                return Ok(false);
            }
            Ok(true)
        };

        if send(Some("MarkdownV2"))? { return Ok(()); }
        if send(Some("Markdown"))? { return Ok(()); }
        if send(None)? { return Ok(()); }
        
        Ok(())
    }
}
