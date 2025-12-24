// Run with: cargo run --features "macros"
// Note: This example uses "core" features only. No server/runtime required.

use radkit::agent::LlmFunction;
use radkit::macros::LLMOutput;
use radkit::models::providers::OpenAILlm;
use radkit::models::Thread;
use schemars::JsonSchema;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, LLMOutput, JsonSchema)]
struct SentimentAnalysis {
    #[serde(rename = "sentimentScore")]
    sentiment_score: f64, // -1.0 to 1.0
    reasoning: String,
    keywords: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup LLM (Stateless)
    // export OPENAI_API_KEY=...
    let llm = OpenAILlm::from_env("gpt-4o")?;

    // 2. Define the Function
    // We want structured output for sentiment analysis.
    let analyzer = LlmFunction::<SentimentAnalysis>::new_with_system_instructions(
        llm,
        "You are an expert sentiment analyzer. Extract precise scores.",
    );

    // 3. Create Input (Immutable Thread)
    let thread = Thread::from_user(
        "I honestly can't believe how good this pizza is, but the service was terrible.",
    );

    println!("Analyzing: {}", thread.last_event().unwrap().content.text().unwrap());

    // 4. Run (Stateless execution)
    let result = analyzer.run(thread).await?;

    println!("Score: {}", result.sentiment_score);
    println!("Reasoning: {}", result.reasoning);
    println!("Keywords: {:?}", result.keywords);

    Ok(())
}
