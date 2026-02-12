use redclaw::config::{Config, AgentsConfig, AgentDefaults, ProvidersConfig, ToolsConfig, WebToolsConfig, WebSearchConfig, ProviderConfig};
use redclaw::agent::Agent;
use redclaw::agent::llm::LLMClient;
use redclaw::tools::registry::ToolRegistry;

#[test]
fn test_agent_initialization() {
    let config = Config {
        agents: AgentsConfig {
            defaults: AgentDefaults {
                workspace: "/tmp/redclaw_test".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                max_tokens: 100,
                temperature: 0.7,
                max_tool_iterations: 5,
            },
        },
        providers: ProvidersConfig {
            openai: Some(ProviderConfig {
                api_key: "fake-key".to_string(),
                api_base: None,
            }),
            gemini: None,
            openrouter: None,
            zhipu: None,
            vllm: None,
        },
        channels: Default::default(),
        tools: ToolsConfig {
            web: WebToolsConfig {
                search: WebSearchConfig {
                    api_key: "".to_string(),
                    max_results: 5,
                },
            },
        },
    };

    let client = LLMClient::new(
        config.providers.openai.as_ref().unwrap(),
        "https://api.openai.com/v1",
        "gpt-3.5-turbo"
    );

    let registry = ToolRegistry::new();
    let agent = Agent::new(&config, client, registry);
    
    // Check if workspace was created
    assert!(std::path::Path::new("/tmp/redclaw_test/memory").exists());
}
