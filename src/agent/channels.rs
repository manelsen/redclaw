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
                .arg("-L")
                .arg("--connect-timeout").arg("10")
                .arg("--max-time").arg("45")
                .arg(url)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match child {
                Ok(mut c) => {
                    if let Some(stdout) = c.stdout.take() {
                        // We read into a String first to be able to debug-print it if JSON fails
                        use std::io::Read;
                        let mut raw_resp = String::new();
                        let mut reader = std::io::BufReader::new(stdout);
                        
                        if let Ok(_) = reader.read_to_string(&mut raw_resp) {
                            match serde_json::from_str::<TgResponse<Vec<TgUpdate>>>(&raw_resp) {
                                Ok(tg_res) => {
                                    if tg_res.ok {
                                        for update in tg_res.result {
                                            offset = update.update_id + 1;
                                            if let Some(msg) = update.message {
                                                if let Err(e) = self.handle_message(agent, msg) {
                                                    eprintln!("Error handling message: {}", e);
                                                }
                                            }
                                        }
                                    } else {
                                        eprintln!("Telegram API Error: {:?}", raw_resp);
                                    }
                                }
                                Err(e) => {
                                    // Only print if it's not empty (curl might return empty on some errors)
                                    if !raw_resp.trim().is_empty() {
                                        eprintln!("Failed to parse Telegram response: {}. Raw: '{}'", e, raw_resp);
                                    }
                                }
                            }
                        }
                    }
                    let _ = c.wait();
                }
                Err(e) => {
                    eprintln!("Failed to spawn curl: {}", e);
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
            .arg("-s").arg("-L").arg("-X").arg("POST")
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
                if let Err(e) = self.send_message(chat_id, &response) {
                    eprintln!("Failed to send message: {}", e);
                }
            }
            Err(e) => {
                println!("  Error:");
                crate::utils::print_box_line(&format!("{}", e));
                crate::utils::print_box_bottom();
                if let Err(e) = self.send_message(chat_id, &format!("Agent Error: {}", e)) {
                    eprintln!("Failed to send error message: {}", e);
                }
            }
        }
        Ok(())
    }

    fn send_raw_message(&self, chat_id: i64, text: &str) -> Result<bool> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        
        let send = |mode: Option<&str>| -> Result<bool> {
            let mut payload = serde_json::json!({
                "chat_id": chat_id,
                "text": text
            });
            if let Some(m) = mode {
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("parse_mode".to_string(), serde_json::json!(m));
                }
            }

            let mut child = Command::new("curl")
                .arg("-s")
                .arg("-L")
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

        if send(Some("MarkdownV2"))? { return Ok(true); }
        if send(Some("Markdown"))? { return Ok(true); }
        if send(None)? { return Ok(true); }
        
        Ok(false)
    }

    fn send_message(&self, chat_id: i64, text: &str) -> Result<()> {
        let max_chars = 4000;
        let mut current_text = text;

        if current_text.chars().count() <= max_chars {
            self.send_raw_message(chat_id, current_text)?;
            return Ok(());
        }

        while !current_text.is_empty() {
            let mut end_idx = current_text.len();
            if current_text.chars().count() > max_chars {
                end_idx = current_text.char_indices().map(|(i, _)| i).nth(max_chars).unwrap_or(current_text.len());
                
                if let Some(last_newline) = current_text[..end_idx].rfind('\n') {
                    if last_newline > (max_chars * 3 / 4) {
                        end_idx = last_newline;
                    }
                }
            }

            let chunk = current_text[..end_idx].trim();
            if !chunk.is_empty() {
                self.send_raw_message(chat_id, chunk)?;
            }
            
            current_text = current_text[end_idx..].trim_start();
        }

        Ok(())
    }
}
