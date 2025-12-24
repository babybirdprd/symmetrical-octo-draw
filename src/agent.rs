use radkit::{
    agent::{LlmWorker, Agent},
    models::{Content, Thread},
    tools::ToolResult,
    macros::tool,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::time::Duration;
use dioxus::prelude::*;
use serde_json::json;

use crate::model::{Action, Shape, ShapeType};
#[cfg(feature = "server")]
use crate::server_state::{AGENT_CHANNEL, AGENT_HISTORY, AGENT_CONFIG};
use crate::model::{AgentConfig, AgentProvider};

// --- DuckDuckGo Tool ---

#[derive(Deserialize, JsonSchema)]
struct SearchArgs {
    query: String,
}

/// Searches the web for a topic.
#[tool(description = "Searches the web for a topic.")]
async fn search(args: SearchArgs) -> ToolResult {
    let topic = args.query;
    let client = reqwest::Client::new();
    let url = format!("https://html.duckduckgo.com/html/?q={}", topic);

    if let Ok(resp) = client.get(&url).header("User-Agent", "Mozilla/5.0").send().await {
        if let Ok(text) = resp.text().await {
            let lower = text.to_lowercase();
            let mut results = String::new();
            if lower.contains("safety") { results.push_str("Found concept: Safety. "); }
            if lower.contains("performance") { results.push_str("Found concept: Performance. "); }
            if lower.contains("concurrency") { results.push_str("Found concept: Concurrency. "); }
            if results.is_empty() {
                results = format!("I found the following content for '{}':\n\n{}\n\n(Note: This is a simple scrape. No specific keywords like 'safety' or 'performance' were explicitly tagged in the simplified parser, but the text is available for analysis.)", topic, text.chars().take(500).collect::<String>());
            }
            return ToolResult::success(json!({ "result": results }));
        }
    }
    ToolResult::error("Failed to perform search")
}

// --- Canvas Tool ---

#[derive(Deserialize, JsonSchema)]
struct DrawArgs {
    shape_type: String, // "circle", "rectangle", "line"
    color: String,
    position_hint: String, // "node 1", "node 2", etc.
}

/// Draws a shape on the canvas.
#[tool(description = "Draws a shape on the canvas.")]
async fn draw_shape(args: DrawArgs) -> ToolResult {
    #[cfg(feature = "server")]
    {
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

        return ToolResult::success(json!({ "result": "Drawn shape" }));
    }
    #[cfg(not(feature = "server"))]
    ToolResult::error("Server only")
}

#[derive(Deserialize, JsonSchema)]
struct WipeArgs {}

/// Wipes the canvas.
#[tool(description = "Wipes the canvas.")]
async fn wipe_board(_args: WipeArgs) -> ToolResult {
    #[cfg(feature = "server")]
    {
        if let Ok(mut history) = AGENT_HISTORY.lock() {
            history.push(Action::Wipe);
        }
        let _ = AGENT_CHANNEL.send(Action::Wipe);
        return ToolResult::success(json!({ "result": "Board wiped" }));
    }
    #[cfg(not(feature = "server"))]
    ToolResult::error("Server only")
}

#[cfg(feature = "server")]
use crate::server_state::AGENT_CONFIG;

#[server]
pub async fn update_agent_config(config: AgentConfig) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
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
        if let Ok(c) = AGENT_CONFIG.lock() {
            return Ok(c.clone());
        }
        Err(ServerFnError::new("Failed to lock config"))
    }
    #[cfg(not(feature = "server"))]
    Err(ServerFnError::new("Not implemented on client"))
}

pub fn init_agent() {
    println!("Agent Initialized");

    #[cfg(feature = "server")]
    tokio::spawn(async move {
        use radkit::models::providers::{OpenAILlm, AnthropicLlm};

        // Wait for server start
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Loop for autonomy
        loop {
            // Read Config
            let config = if let Ok(c) = AGENT_CONFIG.lock() { c.clone() } else { AgentConfig::default() };

            if config.api_key.is_empty() {
                println!("Agent: No API Key set. Waiting...");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            println!("Agent: Starting research cycle with topic: '{}'", config.research_topic);

            // Setup LLM
            let llm_res: Result<Box<dyn radkit::models::Llm>, _> = match config.provider {
                AgentProvider::OpenAI => {
                    std::env::set_var("OPENAI_API_KEY", &config.api_key);
                    OpenAILlm::from_env(&config.model).map(|x| Box::new(x) as Box<dyn radkit::models::Llm>)
                },
                AgentProvider::Anthropic => {
                    std::env::set_var("ANTHROPIC_API_KEY", &config.api_key);
                     match AnthropicLlm::from_env(&config.model) {
                         Ok(llm) => Ok(Box::new(llm) as Box<dyn radkit::models::Llm>),
                         Err(e) => Err(e),
                     }
                }
            };

            if let Ok(llm) = llm_res {
                // Use LlmWorker for simple autonomous loop
                let worker = LlmWorker::<String>::builder(llm)
                    .with_tool(search)
                    .with_tool(draw_shape)
                    .with_tool(wipe_board)
                    .with_system_prompt(&config.system_prompt)
                    .build();

                let prompt = format!("Perform research on: '{}'. Use the search tool to find information, and then use the draw_shape tool to visualize key concepts. Use wipe_board if the board is too cluttered.", config.research_topic);
                let thread = Thread::from_user(&prompt);

                println!("Agent: Asking LLM...");
                match worker.run(thread).await {
                    Ok(response) => println!("Agent: Cycle finished. Response: {}", response),
                    Err(e) => println!("Agent: Error in cycle: {:?}", e),
                }
            } else {
                println!("Agent: Failed to initialize LLM (check model name or key)");
            }

            println!("Agent: Sleeping...");
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });
}

#[cfg(feature = "server")]
#[server]
pub async fn poll_agent(last_index: usize) -> Result<Vec<Action>, ServerFnError> {
    if let Ok(history) = AGENT_HISTORY.lock() {
        if last_index < history.len() {
            return Ok(history[last_index..].to_vec());
        }
    }
    Ok(Vec::new())
}

#[cfg(not(feature = "server"))]
#[server]
pub async fn poll_agent(_last_index: usize) -> Result<Vec<Action>, ServerFnError> {
    Err(ServerFnError::new("Not implemented on client"))
}
