use radkit::tools::{FunctionTool, ToolResult};
use serde::Deserialize;
use crate::model::{Shape, ShapeType};
use serde_json::json;

#[derive(Deserialize)]
pub struct DrawShapeArgs {
    shape_type: String, // "rect", "circle", "line"
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: String,
}

pub fn make_draw_tool() -> FunctionTool {
    FunctionTool::new(
        "draw_shape",
        "Draw a shape (rectangle, circle, or line) on the whiteboard.",
        |args, _ctx| Box::pin(async move {
            let args_value = serde_json::Value::Object(args.into_iter().collect());

            let args: DrawShapeArgs = match serde_json::from_value(args_value) {
                Ok(a) => a,
                Err(e) => return ToolResult::error(format!("Invalid arguments: {}", e)),
            };

            let shape_type = match args.shape_type.as_str() {
                "rect" => ShapeType::Rectangle,
                "circle" => ShapeType::Circle,
                "line" => ShapeType::Line,
                _ => return ToolResult::error(format!("Unknown shape type: {}", args.shape_type)),
            };

            let shape = Shape::new(shape_type, args.x, args.y, args.width, args.height, args.color);
            
            // In a real app, send this action to the Dioxus state via a channel.
            
            ToolResult::success(json!({ "status": "drawn", "shape": shape }))
        })
    )
}

#[derive(Deserialize)]
pub struct WipeBoardArgs {}

pub fn make_wipe_tool() -> FunctionTool {
    FunctionTool::new(
        "wipe_board",
        "Wipe the entire whiteboard clean.",
        |_args, _ctx| Box::pin(async move {
            ToolResult::success(json!({ "status": "wiped" }))
        })
    )
}
