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
