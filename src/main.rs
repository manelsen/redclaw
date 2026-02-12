mod config;
mod agent;
mod tools;
mod utils;

use anyhow::{Result, anyhow};
use redclaw::config::Config;
use redclaw::agent::Agent;
use redclaw::agent::llm::LLMClient;
use redclaw::agent::channels::TelegramBot;
use redclaw::tools::registry::ToolRegistry;
use redclaw::tools::builtin::{ReadFileTool, WriteFileTool, ListDirTool, ExecTool, WebSearchTool, WebFetchTool, SysInfoTool};
use std::env;

struct Args {
    message: Option<String>,
    config: String,
    interactive: bool,
    telegram: bool,
    onboard: bool,
}

fn parse_args() -> Args {
    let mut args = env::args().skip(1);
    let mut parsed = Args {
        message: None,
        config: "config.json".to_string(),
        interactive: false,
        telegram: false,
        onboard: false,
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "onboard" | "--onboard" => parsed.onboard = true,
            "-m" | "--message" => parsed.message = args.next(),
            "-c" | "--config" => {
                if let Some(c) = args.next() {
                    parsed.config = c;
                }
            }
            "-i" | "--interactive" => parsed.interactive = true,
            "-t" | "--telegram" => parsed.telegram = true,
            "-h" | "--help" => {
                println!("RedClaw 🦀 - Ultra-lightweight AI Agent (<2MB RAM)");
                println!("");
                println!("Usage: redclaw [COMMAND] [OPTIONS]");
                println!("");
                println!("Commands:");
                println!("  onboard              Start interactive configuration wizard");
                println!("");
                println!("Options:");
                println!("  -m, --message <MSG>  Send a single message to the agent and exit");
                println!("  -c, --config <PATH>  Path to config.json (default: config.json)");
                println!("  -i, --interactive    Start an interactive session in the terminal");
                println!("  -t, --telegram       Run in Telegram Bot mode");
                println!("  -h, --help           Display this help message");
                println!("");
                println!("Examples:");
                println!("  ./redclaw onboard");
                println!("  ./redclaw -m \"Hello!\"");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}. Use --help for usage.", arg);
                std::process::exit(1);
            }
        }
    }
    parsed
}

fn run_onboard() -> Result<()> {
    use std::io::{self, Write};
    println!("🚀 Welcome to RedClaw Onboarding!");
    println!("This wizard will help you create a 'config.json' file.\n");

    let ask = |question: &str, default: &str| -> String {
        print!("{} [{}]: ", question, default);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() { default.to_string() } else { input.to_string() }
    };

    let or_key = ask("Enter OpenRouter API Key", "");
    let model = ask("Enter Model Name", "arcee-ai/trinity-large-preview:free").to_lowercase();
    let brave_key = ask("Enter Brave Search API Key (optional)", "");
    let tg_token = ask("Enter Telegram Bot Token (optional)", "");
    let tg_user = ask("Enter your Telegram ID/Username (for safety)", "your_id");

    let config_json = serde_json::json!({
        "agents": {
            "defaults": {
                "workspace": "./workspace",
                "model": model,
                "max_tokens": 4096,
                "temperature": 0.7,
                "max_tool_iterations": 10
            }
        },
        "providers": {
            "openrouter": {
                "api_key": or_key,
                "api_base": "https://openrouter.ai/api/v1"
            },
            "openai": {
                "api_key": "",
                "api_base": ""
            },
            "gemini": {
                "api_key": "",
                "api_base": ""
            }
        },
        "tools": {
            "web": {
                "search": {
                    "api_key": brave_key,
                    "max_results": 5
                }
            }
        },
        "channels": {
            "telegram": {
                "enabled": !tg_token.is_empty(),
                "token": tg_token,
                "allow_from": if tg_user == "your_id" { vec![] } else { vec![tg_user] }
            }
        }
    });

    let file = std::fs::File::create("config.json")?;
    serde_json::to_writer_pretty(file, &config_json)?;
    
    println!("\n✅ config.json created successfully!");
    println!("You can now run: ./redclaw -i");
    Ok(())
}

fn main() -> Result<()> {
    let args = parse_args();

    if args.onboard {
        return run_onboard();
    }

    let config = Config::load(&args.config)
        .map_err(|e| anyhow!("Failed to load config from {}: {}", args.config, e))?;

    // Priority: OpenRouter -> OpenAI -> Gemini -> Others
    let provider_config = config.providers.openrouter.as_ref()
        .or(config.providers.openai.as_ref())
        .or(config.providers.gemini.as_ref())
        .or(config.providers.zhipu.as_ref())
        .or(config.providers.vllm.as_ref())
        .ok_or_else(|| anyhow!("No LLM provider configured in {}. See config.example.json", args.config))?;

    let default_base = if config.providers.openrouter.is_some() {
        "https://openrouter.ai/api/v1"
    } else if config.providers.gemini.is_some() {
        "https://generativelanguage.googleapis.com/v1beta/openai"
    } else if config.providers.zhipu.is_some() {
        "https://openapi.zhipuai.cn/api/paas/v4"
    } else {
        "https://api.openai.com/v1"
    };

    let client = LLMClient::new(
        provider_config,
        default_base,
        &config.agents.defaults.model
    );

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(ReadFileTool));
    registry.register(Box::new(WriteFileTool));
    registry.register(Box::new(ListDirTool));
    registry.register(Box::new(ExecTool {
        working_dir: config.workspace_path().to_string_lossy().to_string(),
    }));
    registry.register(Box::new(WebSearchTool {
        api_key: config.tools.web.search.api_key.clone(),
        max_results: config.tools.web.search.max_results,
    }));
    registry.register(Box::new(WebFetchTool));
    registry.register(Box::new(SysInfoTool));

    let mut agent = Agent::new(&config, client, registry);

    if args.telegram {
        let tg_cfg = config.channels.telegram.as_ref()
            .ok_or_else(|| anyhow!("Telegram not configured in {}", args.config))?;
        if !tg_cfg.enabled {
            return Err(anyhow!("Telegram is disabled in config"));
        }
        let bot = TelegramBot::new(tg_cfg.token.clone(), tg_cfg.allow_from.clone());
        bot.run(&mut agent)?;
        } else if let Some(msg) = args.message {
            let response = agent.run(&msg)?;
            crate::utils::print_box_top("Claw");
            crate::utils::print_box_line(&response);
            crate::utils::print_box_bottom();
        } else if args.interactive {
    
            use std::io::{self, Write};
            println!("RedClaw Interactive Mode");
            loop {
                print!("╭─ Input: ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim();
                if input.is_empty() { continue; }
                if input == "exit" || input == "quit" { break; }
                
            match agent.run(input) {
                Ok(response) => {
                    println!("\n  Claw:");
                    crate::utils::print_box_line(&response);
                    crate::utils::print_box_bottom();
                    println!("");
                },
                Err(e) => {
                    println!("\n  Error:");
                    crate::utils::print_box_line(&format!("{}", e));
                    crate::utils::print_box_bottom();
                    println!("");
                }
            }
            }
        }
     else {
        println!("No mode specified. Use --help for usage info.");
    }
    Ok(())
}
