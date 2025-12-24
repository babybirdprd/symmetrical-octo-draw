// Run with: cargo run --features "macros"

use radkit::agent::LlmWorker;
use radkit::macros::tool;
use radkit::models::providers::OpenAILlm;
use radkit::models::Thread;
use radkit::tools::{FunctionTool, ToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

// --- Method 1: The Macro (Recommended) ---

#[derive(Deserialize, JsonSchema)]
struct CalculatorArgs {
    a: i32,
    b: i32,
}

/// Adds two numbers together.
#[tool]
async fn add(args: CalculatorArgs) -> ToolResult {
    let result = args.a + args.b;
    ToolResult::success(json!({ "result": result }))
}

// --- Method 2: Manual (For dynamic tools) ---

fn make_subtract_tool() -> FunctionTool {
    FunctionTool::new(
        "subtract",
        "Subtracts b from a",
        |args, _ctx| Box::pin(async move {
            let a = args["a"].as_i64().unwrap_or(0);
            let b = args["b"].as_i64().unwrap_or(0);
            ToolResult::success(json!({ "result": a - b }))
        })
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm = OpenAILlm::from_env("gpt-4o")?;

    // We want the final answer as a String
    // Note: Use Thread as the generic if you want the full conversation history instead
    let worker = LlmWorker::<String>::builder(llm)
        .with_tool(add)           // Macro tool
        .with_tool(make_subtract_tool()) // Manual tool
        .build();

    let thread = Thread::from_user("What is 10 plus 5, and then minus 3?");

    // The worker will:
    // 1. Call 'add(10, 5)' -> 15
    // 2. Call 'subtract(15, 3)' -> 12
    // 3. Return "12"
    let response = worker.run(thread).await?;

    println!("Final Answer: {}", response);

    Ok(())
}
