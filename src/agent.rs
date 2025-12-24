use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;

use crate::model::{Action, Shape, ShapeType};
use crate::model::{AgentConfig, AgentProvider};

// ============================================================================
// SERVER-ONLY MODULE
// All radkit/agent code is compiled only for the server target
// ============================================================================
#[cfg(feature = "server")]
mod server_agent {
    use super::*;
    use crate::server_state::{AGENT_CHANNEL, AGENT_CONFIG, AGENT_HISTORY};
    use radkit::models::providers::{
        AnthropicLlm, DeepSeekLlm, GeminiLlm, GroqLlm, OpenAILlm, OpenRouterLlm,
    };
    use radkit::{agent::LlmWorker, macros::tool, models::Thread, tools::ToolResult};
    use schemars::JsonSchema;
    use std::time::Duration;

    // --- DuckDuckGo Search Tool ---
    #[derive(Deserialize, JsonSchema)]
    struct SearchArgs {
        query: String,
    }

    #[tool(description = "Searches the web for a topic.")]
    async fn search(args: SearchArgs) -> ToolResult {
        let topic = args.query;
        let client = reqwest::Client::new();
        let url = format!("https://html.duckduckgo.com/html/?q={}", topic);

        if let Ok(resp) = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
        {
            if let Ok(text) = resp.text().await {
                let lower = text.to_lowercase();
                let mut results = String::new();
                if lower.contains("safety") {
                    results.push_str("Found concept: Safety. ");
                }
                if lower.contains("performance") {
                    results.push_str("Found concept: Performance. ");
                }
                if lower.contains("concurrency") {
                    results.push_str("Found concept: Concurrency. ");
                }
                if results.is_empty() {
                    results = format!(
                        "I found the following content for '{}':\n\n{}\n\n(Simplified scrape)",
                        topic,
                        text.chars().take(500).collect::<String>()
                    );
                }
                return ToolResult::success(json!({ "result": results }));
            }
        }
        ToolResult::error("Failed to perform search")
    }

    // --- Canvas Draw Tool ---
    #[derive(Deserialize, JsonSchema)]
    struct DrawArgs {
        shape_type: String,
        color: String,
        position_hint: String,
    }

    #[tool(description = "Draws a shape on the canvas.")]
    async fn draw_shape(args: DrawArgs) -> ToolResult {
        let shape_type = match args.shape_type.to_lowercase().as_str() {
            "circle" => ShapeType::Circle,
            "line" => ShapeType::Line,
            _ => ShapeType::Rectangle,
        };

        let x = match args.position_hint.to_lowercase().as_str() {
            "node 1" => 100.0,
            "node 2" => 300.0,
            "node 3" => 500.0,
            _ => 200.0,
        };
        let y = 200.0;

        let shape = Shape::new(shape_type, x, y, 100.0, 100.0, args.color.clone());
        let action = Action::Draw(shape);

        if let Ok(mut history) = AGENT_HISTORY.lock() {
            history.push(action.clone());
        }
        let _ = AGENT_CHANNEL.send(action);

        ToolResult::success(json!({ "result": "Drawn shape" }))
    }

    // --- Canvas Wipe Tool ---
    #[derive(Deserialize, JsonSchema)]
    struct WipeArgs {}

    #[tool(description = "Wipes the canvas.")]
    async fn wipe_board(_args: WipeArgs) -> ToolResult {
        if let Ok(mut history) = AGENT_HISTORY.lock() {
            history.push(Action::Wipe);
        }
        let _ = AGENT_CHANNEL.send(Action::Wipe);
        ToolResult::success(json!({ "result": "Board wiped" }))
    }

    // --- Agent Loop ---
    pub fn start_agent_loop() {
        // Spawn in a new thread with its own Tokio runtime to avoid "no reactor" panic
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async move {
                use radkit::models::providers::{AnthropicLlm, OpenAILlm};

                // Wait for server start
                tokio::time::sleep(Duration::from_secs(5)).await;

                loop {
                    let config = if let Ok(c) = AGENT_CONFIG.lock() {
                        c.clone()
                    } else {
                        AgentConfig::default()
                    };

                    if config.api_key.is_empty() {
                        println!("Agent: No API Key set. Waiting...");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }

                    if config.model.is_empty() {
                        println!("Agent: No model set. Waiting...");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }

                    println!(
                        "Agent: Starting research cycle with provider:{:?} model:'{}' topic:'{}'",
                        config.provider, config.model, config.research_topic
                    );

                    let prompt = format!(
                        "Perform research on: '{}'. Use the search tool to find information, \
                     then use the draw_shape tool to visualize key concepts. \
                     Use wipe_board if the board is too cluttered.",
                        config.research_topic
                    );

                    let result = match config.provider {
                        AgentProvider::OpenAI => {
                            std::env::set_var("OPENAI_API_KEY", &config.api_key);
                            match OpenAILlm::from_env(&config.model) {
                                Ok(llm) => {
                                    let worker = LlmWorker::<String>::builder(llm)
                                        .with_tool(search)
                                        .with_tool(draw_shape)
                                        .with_tool(wipe_board)
                                        .build();
                                    let thread = Thread::from_user(&prompt);
                                    worker.run(thread).await
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                        AgentProvider::Anthropic => {
                            std::env::set_var("ANTHROPIC_API_KEY", &config.api_key);
                            match AnthropicLlm::from_env(&config.model) {
                                Ok(llm) => {
                                    let worker = LlmWorker::<String>::builder(llm)
                                        .with_tool(search)
                                        .with_tool(draw_shape)
                                        .with_tool(wipe_board)
                                        .build();
                                    let thread = Thread::from_user(&prompt);
                                    worker.run(thread).await
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                        AgentProvider::OpenRouter => {
                            std::env::set_var("OPENROUTER_API_KEY", &config.api_key);
                            match OpenRouterLlm::from_env(&config.model) {
                                Ok(llm) => {
                                    let worker = LlmWorker::<String>::builder(llm)
                                        .with_tool(search)
                                        .with_tool(draw_shape)
                                        .with_tool(wipe_board)
                                        .build();
                                    let thread = Thread::from_user(&prompt);
                                    worker.run(thread).await
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                        AgentProvider::Gemini => {
                            std::env::set_var("GEMINI_API_KEY", &config.api_key);
                            match GeminiLlm::from_env(&config.model) {
                                Ok(llm) => {
                                    let worker = LlmWorker::<String>::builder(llm)
                                        .with_tool(search)
                                        .with_tool(draw_shape)
                                        .with_tool(wipe_board)
                                        .build();
                                    let thread = Thread::from_user(&prompt);
                                    worker.run(thread).await
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                        AgentProvider::Groq => {
                            std::env::set_var("GROQ_API_KEY", &config.api_key);
                            match GroqLlm::from_env(&config.model) {
                                Ok(llm) => {
                                    let worker = LlmWorker::<String>::builder(llm)
                                        .with_tool(search)
                                        .with_tool(draw_shape)
                                        .with_tool(wipe_board)
                                        .build();
                                    let thread = Thread::from_user(&prompt);
                                    worker.run(thread).await
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                        AgentProvider::DeepSeek => {
                            std::env::set_var("DEEPSEEK_API_KEY", &config.api_key);
                            match DeepSeekLlm::from_env(&config.model) {
                                Ok(llm) => {
                                    let worker = LlmWorker::<String>::builder(llm)
                                        .with_tool(search)
                                        .with_tool(draw_shape)
                                        .with_tool(wipe_board)
                                        .build();
                                    let thread = Thread::from_user(&prompt);
                                    worker.run(thread).await
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                    };

                    match result {
                        Ok(response) => println!("Agent: Cycle finished. Response: {}", response),
                        Err(e) => println!("Agent: Error in cycle: {:?}", e),
                    }

                    println!("Agent: Sleeping...");
                    tokio::time::sleep(Duration::from_secs(60)).await;
                } // end of loop
            }); // end of async block and block_on()
        }); // end of thread::spawn closure
    } // end of start_agent_loop
} // end of server_agent mod

// ============================================================================
// PUBLIC API (works on both server and client)
// ============================================================================

/// Initialize the agent (only does something on server)
pub fn init_agent() {
    println!("Agent Initialized");

    #[cfg(feature = "server")]
    server_agent::start_agent_loop();
}

// ============================================================================
// SERVER FUNCTIONS (Dioxus fullstack RPC)
// ============================================================================

#[server]
pub async fn update_agent_config(config: AgentConfig) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::server_state::AGENT_CONFIG;
        if let Ok(mut c) = AGENT_CONFIG.lock() {
            *c = config;
        }
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    Err(ServerFnError::new("Not implemented on client"))
}

#[server]
pub async fn get_agent_config() -> Result<AgentConfig, ServerFnError> {
    #[cfg(feature = "server")]
    {
        use crate::server_state::AGENT_CONFIG;
        if let Ok(c) = AGENT_CONFIG.lock() {
            return Ok(c.clone());
        }
        Err(ServerFnError::new("Failed to lock config"))
    }
    #[cfg(not(feature = "server"))]
    Err(ServerFnError::new("Not implemented on client"))
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
    Err(ServerFnError::new("Not implemented on client"))
}
