use radkit::tools::{FunctionTool, ToolResult};
use serde::Deserialize;
use crate::model::{Action, Shape, ShapeType};
use crate::server_state::{AGENT_CHANNEL, AGENT_HISTORY};
use serde_json::json;

#[derive(Deserialize)]
pub struct DrawShapeArgs {
    /// Type of shape: "rectangle", "circle", or "line"
    shape_type: String,
    /// X position (0-800)
    x: Option<f64>,
    /// Y position (0-500)
    y: Option<f64>,
    /// Width of shape
    width: Option<f64>,
    /// Height of shape
    height: Option<f64>,
    /// Color (e.g. "red", "blue", "#FF5733")
    color: String,
    /// Label for this shape (what concept it represents)
    label: Option<String>,
}

pub fn make_draw_tool() -> FunctionTool {
    FunctionTool::new(
        "draw_shape",
        "Draw a labeled shape on the presentation canvas. Use this to create visual diagrams. Each shape can represent a concept, fact, or category from your research.",
        |args, _ctx| Box::pin(async move {
            let args_value = serde_json::Value::Object(args.into_iter().collect());

            let args: DrawShapeArgs = match serde_json::from_value(args_value) {
                Ok(a) => a,
                Err(e) => return ToolResult::error(format!("Invalid arguments: {}", e)),
            };

            let shape_type = match args.shape_type.to_lowercase().as_str() {
                "circle" => ShapeType::Circle,
                "line" => ShapeType::Line,
                _ => ShapeType::Rectangle,
            };

            let x = args.x.unwrap_or(100.0);
            let y = args.y.unwrap_or(100.0);
            let width = args.width.unwrap_or(150.0);
            let height = args.height.unwrap_or(100.0);

            let label = args.label.clone().unwrap_or_default();
            let shape = Shape::new(shape_type, x, y, width, height, args.color.clone(), label.clone());
            let action = Action::Draw(shape);
            
            println!("Tool: Drawing {} '{}' at ({}, {})", args.shape_type, label, x, y);

            // Send to channel AND history
            if let Ok(mut history) = AGENT_HISTORY.lock() {
                let history_len = history.len();
                history.push(action.clone());
                // Only used for return value
                let _ = history_len;
            }
            let history_len = AGENT_HISTORY.lock().map(|h| h.len()).unwrap_or(0);
            let _ = AGENT_CHANNEL.send(action);
            
            ToolResult::success(json!({ 
                "status": "drawn", 
                "label": label,
                "position": { "x": x, "y": y },
                "shape_id": history_len
            }))
        })
    )
}

#[derive(Deserialize)]
pub struct WipeBoardArgs {}

pub fn make_wipe_tool() -> FunctionTool {
    FunctionTool::new(
        "wipe_board",
        "Clear all shapes from the canvas. Use before creating a new diagram or when the board is cluttered.",
        |_args, _ctx| Box::pin(async move {
            println!("Tool: Wiping board");
            
            if let Ok(mut history) = AGENT_HISTORY.lock() {
                history.push(Action::Wipe);
            }
            let _ = AGENT_CHANNEL.send(Action::Wipe);
            
            ToolResult::success(json!({ "status": "cleared" }))
        })
    )
}
