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
}

impl Shape {
    pub fn new(
        shape_type: ShapeType,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            shape_type,
            x,
            y,
            width,
            height,
            color,
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

/// Supported LLM providers
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum AgentProvider {
    #[default]
    OpenAI,
    Anthropic,
    Gemini,
    Groq,
    OpenRouter,
    Custom, // For any OpenAI-compatible endpoint
}

impl AgentProvider {
    /// Returns all available providers for UI display
    pub fn all() -> Vec<AgentProvider> {
        vec![
            AgentProvider::OpenAI,
            AgentProvider::Anthropic,
            AgentProvider::Gemini,
            AgentProvider::Groq,
            AgentProvider::OpenRouter,
            AgentProvider::Custom,
        ]
    }

    /// Returns the display name for the provider
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentProvider::OpenAI => "OpenAI",
            AgentProvider::Anthropic => "Anthropic",
            AgentProvider::Gemini => "Google Gemini",
            AgentProvider::Groq => "Groq",
            AgentProvider::OpenRouter => "OpenRouter",
            AgentProvider::Custom => "Custom (OpenAI-compatible)",
        }
    }

    /// Returns the default model for each provider
    pub fn default_model(&self) -> &'static str {
        match self {
            AgentProvider::OpenAI => "gpt-4o",
            AgentProvider::Anthropic => "claude-3-5-sonnet-20241022",
            AgentProvider::Gemini => "gemini-pro",
            AgentProvider::Groq => "llama-3.1-70b-versatile",
            AgentProvider::OpenRouter => "openai/gpt-4o",
            AgentProvider::Custom => "gpt-4o",
        }
    }

    /// Returns whether this provider supports/requires a base_url
    pub fn supports_base_url(&self) -> bool {
        matches!(
            self,
            AgentProvider::OpenAI | AgentProvider::Custom | AgentProvider::OpenRouter
        )
    }
}

/// Configuration for the AI agent
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    pub provider: AgentProvider,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>, // For OpenAI-compatible endpoints (Azure, local LLMs, etc.)
    pub system_prompt: String,
    pub research_topic: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: AgentProvider::OpenAI,
            model: String::new(),  // Empty by default - user must enter
            api_key: String::new(),
            base_url: None,
            system_prompt: "You are a research agent. Use the tools available to research the topic and visualize the findings on the board.".to_string(),
            research_topic: "Rust programming language state management".to_string(),
        }
    }
}
