use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Line,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Shape {
    pub id: Uuid,
    pub shape_type: ShapeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: String,
    pub label: String,
}

impl Shape {
    pub fn new(
        shape_type: ShapeType,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: String,
        label: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            shape_type,
            x,
            y,
            width,
            height,
            color,
            label,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Board {
    pub shapes: Vec<Shape>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Draw(Shape),
    Wipe,
    NewBoard,
}

/// Supported LLM providers (matches radkit::models::providers)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum AgentProvider {
    #[default]
    OpenAI,
    Anthropic,
    OpenRouter,
    Gemini,
    Grok,
    DeepSeek,
}

impl AgentProvider {
    pub fn all() -> Vec<AgentProvider> {
        vec![
            AgentProvider::OpenAI,
            AgentProvider::Anthropic,
            AgentProvider::OpenRouter,
            AgentProvider::Gemini,
            AgentProvider::Grok,
            AgentProvider::DeepSeek,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AgentProvider::OpenAI => "OpenAI",
            AgentProvider::Anthropic => "Anthropic",
            AgentProvider::OpenRouter => "OpenRouter",
            AgentProvider::Gemini => "Google Gemini",
            AgentProvider::Grok => "Grok",
            AgentProvider::DeepSeek => "DeepSeek",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            AgentProvider::OpenAI => "gpt-4o",
            AgentProvider::Anthropic => "claude-3-5-sonnet-20241022",
            AgentProvider::OpenRouter => "anthropic/claude-3.5-sonnet",
            AgentProvider::Gemini => "gemini-1.5-flash-latest",
            AgentProvider::Grok => "llama-3.1-70b-versatile",
            AgentProvider::DeepSeek => "deepseek-chat",
        }
    }
}

/// Configuration for the AI agent
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    pub provider: AgentProvider,
    pub model: String,
    pub api_key: String,
    pub system_prompt: String,
    pub research_topic: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: AgentProvider::OpenAI,
            model: String::new(),
            api_key: String::new(),
            system_prompt: "You are a research agent. Use the tools available to research the topic and visualize the findings on the board.".to_string(),
            research_topic: String::new(),
        }
    }
}
