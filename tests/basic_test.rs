#[cfg(test)]
mod tests {
    use redclaw::config::{Config, AgentsConfig, AgentDefaults, ProvidersConfig, ToolsConfig, WebToolsConfig, WebSearchConfig};
    use redclaw::agent::Agent;
    use redclaw::agent::llm::LLMClient;
    use redclaw::tools::registry::ToolRegistry;
    use redclaw::config::ProviderConfig;

    #[test]
    fn test_config_instantiation() {
        let config = Config {
            agents: AgentsConfig {
                defaults: AgentDefaults {
                    workspace: "/tmp/redclaw".to_string(),
                    model: "test-model".to_string(),
                    max_tokens: 100,
                    temperature: 0.5,
                    max_tool_iterations: 5,
                },
            },
            providers: ProvidersConfig {
                openai: Some(ProviderConfig {
                    api_key: "test-key".to_string(),
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
                        api_key: "test-search-key".to_string(),
                        max_results: 5,
                    },
                },
            },
        };
        assert_eq!(config.agents.defaults.model, "test-model");
    }
}
