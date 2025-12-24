use radkit::{
    agent::{Agent, OnRequestResult, SkillHandler},
    models::Content,
    runtime::{AgentRuntime, context::State, context::ProgressSender},
    errors::AgentError,
};
use async_trait::async_trait;
use crate::model::{Action, Shape, ShapeType};
use std::time::Duration;
use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::server_state::{AGENT_CHANNEL, AGENT_HISTORY};

#[derive(Clone)]
pub struct ResearchAgent;

#[async_trait]
impl SkillHandler for ResearchAgent {
    async fn on_request(
        &self,
        _state: &mut State,
        _progress: &ProgressSender,
        _runtime: &dyn AgentRuntime,
        content: Content,
    ) -> Result<OnRequestResult, AgentError> {
        println!("Agent received request: {:?}", content);
        Ok(OnRequestResult::Completed {
            message: Some(Content::from_text("I have received your request.")),
            artifacts: vec![],
        })
    }
}

pub fn init_agent() {
    println!("Agent Initialized");

    #[cfg(feature = "server")]
    tokio::spawn(async move {
        // Wait for server start
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Loop for autonomy
        loop {
            println!("Agent: Starting research cycle...");

            // 1. Research Topic
            let topic = "Rust programming language state management";
            println!("Agent: Searching for '{}'...", topic);

            let mut found_keywords = Vec::new();

            // Inline Search Logic
            let client = reqwest::Client::new();
            let url = format!("https://html.duckduckgo.com/html/?q={}", topic);
            if let Ok(resp) = client.get(&url).header("User-Agent", "Mozilla/5.0").send().await {
                if let Ok(text) = resp.text().await {
                    let lower = text.to_lowercase();
                    if lower.contains("safety") { found_keywords.push("Safety"); }
                    if lower.contains("performance") { found_keywords.push("Performance"); }
                    if lower.contains("concurrency") { found_keywords.push("Concurrency"); }
                    println!("Agent: Found keywords: {:?}", found_keywords);
                }
            }

            // Helper to broadcast and save action
            let send_action = |action: Action| {
                if let Ok(mut history) = AGENT_HISTORY.lock() {
                    history.push(action.clone());
                }
                let _ = AGENT_CHANNEL.send(action);
            };

            // 2. Wipe Board
            println!("Agent: Wiping board...");
            send_action(Action::Wipe);
            tokio::time::sleep(Duration::from_secs(2)).await;

            // 3. Visualize Findings
            println!("Agent: Drawing results based on research...");
            let start_x = 100.0;
            let mut current_x = start_x;

            // Draw Title Node
            let title_shape = Shape::new(ShapeType::Rectangle, 300.0, 50.0, 200.0, 60.0, "lightgray".to_string());
            send_action(Action::Draw(title_shape));
            tokio::time::sleep(Duration::from_millis(500)).await;

            for (_i, keyword) in found_keywords.iter().enumerate() {
                let color = match *keyword {
                    "Safety" => "green",
                    "Performance" => "blue",
                    "Concurrency" => "orange",
                    _ => "gray",
                };

                // Draw Node
                let shape = Shape::new(ShapeType::Circle, current_x, 200.0, 100.0, 100.0, color.to_string());
                send_action(Action::Draw(shape));
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Draw Connection to Title
                let line = Shape::new(ShapeType::Line, 400.0, 110.0, current_x + 50.0 - 400.0, 200.0 - 110.0, "black".to_string());
                send_action(Action::Draw(line));
                tokio::time::sleep(Duration::from_millis(500)).await;

                current_x += 150.0;
            }

            println!("Agent: Cycle complete. Sleeping...");
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
