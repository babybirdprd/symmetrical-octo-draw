// Run with: cargo run --features "runtime, macros"

use async_trait::async_trait;
use radkit::agent::{
    Agent, Artifact, OnInputResult, OnRequestResult, RegisteredSkill, SkillHandler, SkillMetadata,
    SkillSlot,
};
use radkit::errors::AgentResult;
use radkit::models::{Content, Thread};
use radkit::models::providers::OpenAILlm;
use radkit::runtime::context::{ProgressSender, State};
use radkit::runtime::{AgentRuntime, Runtime};
use serde::{Deserialize, Serialize};

// --- 1. Define the Skill Logic ---

struct GreeterSkill;

// The "State Machine" for our multi-turn conversation
#[derive(Serialize, Deserialize)]
enum GreeterSlot {
    WaitingForName,
}

#[async_trait]
impl SkillHandler for GreeterSkill {
    async fn on_request(
        &self,
        _state: &mut State,
        progress: &ProgressSender,
        runtime: &dyn AgentRuntime,
        content: Content,
    ) -> AgentResult<OnRequestResult> {
        let user_text = content.text().unwrap_or_default();
        let user_name = runtime.current_user().user_name;

        progress.send_update("Checking greeting requirements...").await?;

        // Simple logic: If they didn't say their name, ask for it.
        // In a real agent, we might check memory or use an LLM here.

        if user_text.contains("my name is") {
             return Ok(OnRequestResult::Completed {
                message: Some(Content::from_text(format!("Nice to meet you!"))),
                artifacts: vec![],
            });
        }

        // Transition to InputRequired state
        Ok(OnRequestResult::InputRequired {
            message: Content::from_text(format!("Hi {}! What is your full name?", user_name)),
            slot: SkillSlot::new(GreeterSlot::WaitingForName),
        })
    }

    async fn on_input_received(
        &self,
        _state: &mut State,
        _progress: &ProgressSender,
        _runtime: &dyn AgentRuntime,
        content: Content,
    ) -> AgentResult<OnInputResult> {
        // We know we are in "WaitingForName" because that's the only slot we emitted.
        // In complex skills, check `state.task().get_slot()?`.

        let name = content.text().unwrap_or("Stranger");

        // Return an Artifact just to show off
        let badge = Artifact::from_json("badge.json", &serde_json::json!({ "name": name }))?;

        Ok(OnInputResult::Completed {
            message: Some(Content::from_text(format!("Welcome, {}! I made you a badge.", name))),
            artifacts: vec![badge],
        })
    }
}

// Metadata is required for the Agent to "know" about the skill
impl RegisteredSkill for GreeterSkill {
    fn metadata() -> &'static SkillMetadata {
        &SkillMetadata {
            id: "greeter",
            name: "Greeter Skill",
            description: "Welcomes users and makes badges",
            tags: &["utility"],
            examples: &["Hello", "Hi"],
            input_modes: &["text/plain"],
            output_modes: &["text/plain"],
        }
    }
}

// --- 2. Run the Server ---

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup LLM (Required by Runtime for orchestration, even if skill doesn't use it)
    let llm = OpenAILlm::from_env("gpt-4o")?;

    // 2. Build Agent Definition
    let agent_def = Agent::builder()
        .with_name("GreeterBot")
        .with_description("A friendly bot")
        .with_skill(GreeterSkill)
        .build();

    // 3. Configure Runtime
    let runtime = Runtime::builder(agent_def, llm)
        .base_url("http://localhost:3000") // Important for A2A callbacks
        .build();

    // 4. Start Server
    println!("Starting Agent on 0.0.0.0:3000");
    runtime.serve("0.0.0.0:3000").await?;

    Ok(())
}
