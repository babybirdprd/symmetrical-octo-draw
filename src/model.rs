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
    pub fn new(shape_type: ShapeType, x: f64, y: f64, width: f64, height: f64, color: String) -> Self {
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum AgentProvider {
    #[default]
    OpenAI,
    Anthropic,
}

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
            model: "gpt-4o".to_string(),
            api_key: String::new(),
            system_prompt: "You are a research agent. Use the tools available to research the topic and visualize the findings on the board.".to_string(),
            research_topic: "Rust programming language state management".to_string(),
        }
    }
}
