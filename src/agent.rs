use crate::model::{Action, AgentConfig, AgentProvider};
use dioxus::prelude::*;

// ============================================================================
// SERVER-ONLY MODULE - Agent Loop
// ============================================================================
#[cfg(feature = "server")]
mod server_agent {
    use super::*;
    use crate::server_state::{AGENT_CONFIG, AGENT_HISTORY};
    use crate::tools::{
        board::{make_draw_tool, make_wipe_tool},
        ddg::make_ddg_tool,
    };
    use radkit::agent::LlmWorker;
    use radkit::models::providers::{
        AnthropicLlm, DeepSeekLlm, GeminiLlm, GrokLlm, OpenAILlm, OpenRouterLlm,
    };
    use radkit::models::Thread;
    use std::time::Duration;

    pub fn start_agent_loop() {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                // Wait for server start
                tokio::time::sleep(Duration::from_secs(3)).await;
                println!("Agent: Loop started");

                loop {
                    let config = AGENT_CONFIG.lock()
                        .map(|c| c.clone())
                        .unwrap_or_default();

                    // Skip if not configured
                    if config.api_key.is_empty() || config.model.is_empty() || config.research_topic.is_empty() {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        continue;
                    }

                    println!(
                        "Agent: Starting cycle - provider:{:?} model:'{}' topic:'{}'",
                        config.provider, config.model, config.research_topic
                    );

                    // Build prompt with system instructions
                    let prompt = format!(
                        "{}\n\nResearch topic: '{}'\n\nUse web_search to find information, then use draw_shape to create a visual presentation with multiple shapes representing key concepts. Use wipe_board first if the canvas has old content.",
                        config.system_prompt,
                        config.research_topic
                    );

                    // Create tools from the tools/ module
                    let search_tool = make_ddg_tool();
                    let draw_tool = make_draw_tool();
                    let wipe_tool = make_wipe_tool();

                    // Run with appropriate provider
                    let result = run_with_provider(&config, &prompt, search_tool, draw_tool, wipe_tool).await;

                    match result {
                        Ok(response) => println!("Agent: Cycle finished. Response: {}", response),
                        Err(e) => println!("Agent: Error in cycle: {:?}", e),
                    }

                    // Clear topic after processing so we don't repeat
                    if let Ok(mut c) = AGENT_CONFIG.lock() {
                        c.research_topic.clear();
                    }

                    println!("Agent: Waiting for next research topic...");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            });
        });
    }

    async fn run_with_provider(
        config: &AgentConfig,
        prompt: &str,
        search_tool: radkit::tools::FunctionTool,
        draw_tool: radkit::tools::FunctionTool,
        wipe_tool: radkit::tools::FunctionTool,
    ) -> Result<String, radkit::errors::AgentError> {
        let thread = Thread::from_user(prompt);

        match config.provider {
            AgentProvider::OpenAI => {
                std::env::set_var("OPENAI_API_KEY", &config.api_key);
                let llm = OpenAILlm::from_env(&config.model)?;
                LlmWorker::<String>::builder(llm)
                    .with_tool(search_tool)
                    .with_tool(draw_tool)
                    .with_tool(wipe_tool)
                    .build()
                    .run(thread)
                    .await
            }
            AgentProvider::Anthropic => {
                std::env::set_var("ANTHROPIC_API_KEY", &config.api_key);
                let llm = AnthropicLlm::from_env(&config.model)?;
                LlmWorker::<String>::builder(llm)
                    .with_tool(search_tool)
                    .with_tool(draw_tool)
                    .with_tool(wipe_tool)
                    .build()
                    .run(thread)
                    .await
            }
            AgentProvider::OpenRouter => {
                std::env::set_var("OPENROUTER_API_KEY", &config.api_key);
                let llm = OpenRouterLlm::from_env(&config.model)?;
                LlmWorker::<String>::builder(llm)
                    .with_tool(search_tool)
                    .with_tool(draw_tool)
                    .with_tool(wipe_tool)
                    .build()
                    .run(thread)
                    .await
            }
            AgentProvider::Gemini => {
                std::env::set_var("GEMINI_API_KEY", &config.api_key);
                let llm = GeminiLlm::from_env(&config.model)?;
                LlmWorker::<String>::builder(llm)
                    .with_tool(search_tool)
                    .with_tool(draw_tool)
                    .with_tool(wipe_tool)
                    .build()
                    .run(thread)
                    .await
            }
            AgentProvider::Grok => {
                std::env::set_var("Grok_API_KEY", &config.api_key);
                let llm = GrokLlm::from_env(&config.model)?;
                LlmWorker::<String>::builder(llm)
                    .with_tool(search_tool)
                    .with_tool(draw_tool)
                    .with_tool(wipe_tool)
                    .build()
                    .run(thread)
                    .await
            }
            AgentProvider::DeepSeek => {
                std::env::set_var("DEEPSEEK_API_KEY", &config.api_key);
                let llm = DeepSeekLlm::from_env(&config.model)?;
                LlmWorker::<String>::builder(llm)
                    .with_tool(search_tool)
                    .with_tool(draw_tool)
                    .with_tool(wipe_tool)
                    .build()
                    .run(thread)
                    .await
            }
        }
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

pub fn init_agent() {
    println!("Agent: Initializing...");
    #[cfg(feature = "server")]
    server_agent::start_agent_loop();
}

// ============================================================================
// SERVER FUNCTIONS (Dioxus RPC)
// ============================================================================

#[server]
pub async fn update_agent_config(config: AgentConfig) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::server_state::AGENT_CONFIG;
        println!(
            "Server: Updating config - provider:{:?} model:{} topic:{}",
            config.provider, config.model, config.research_topic
        );
        if let Ok(mut c) = AGENT_CONFIG.lock() {
            *c = config;
        }
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    Err(ServerFnError::new("Not on server"))
}

#[server]
pub async fn get_agent_config() -> Result<AgentConfig, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::server_state::AGENT_CONFIG;
        if let Ok(c) = AGENT_CONFIG.lock() {
            return Ok(c.clone());
        }
        Ok(AgentConfig::default())
    }
    #[cfg(not(feature = "server"))]
    Err(ServerFnError::new("Not on server"))
}

#[server]
pub async fn poll_agent(last_index: usize) -> Result<Vec<Action>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::server_state::AGENT_HISTORY;
        if let Ok(history) = AGENT_HISTORY.lock() {
            if last_index < history.len() {
                return Ok(history[last_index..].to_vec());
            }
        }
        Ok(Vec::new())
    }
    #[cfg(not(feature = "server"))]
    Err(ServerFnError::new("Not on server"))
}
