use crate::components::Hero;
use crate::state::BoardState;
use crate::model::ShapeType;
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    // Initialize the global state if not already provided (though usually you provide it at App level)
    // For simplicity in this step, we use a local-ish signal approach or context.
    // Ideally, `use_context` if initialized in main.
    // Let's create it here for now if we don't have a global context provider yet.
    let mut state = use_context_provider(|| BoardState::new());

    rsx! {
        div { class: "flex flex-col items-center justify-center min-h-screen bg-gray-100",
            h1 { class: "text-2xl font-bold mb-4", "Agent Excalidraw" }
            
            // Canvas Area
            div { class: "border-2 border-gray-300 bg-white shadow-lg w-[800px] h-[600px] relative",
                svg {
                    width: "100%",
                    height: "100%",
                    view_box: "0 0 800 600",
                    for shape in state.board.read().shapes.iter() {
                        match shape.shape_type {
                            ShapeType::Rectangle => rsx! {
                                rect {
                                    x: "{shape.x}",
                                    y: "{shape.y}",
                                    width: "{shape.width}",
                                    height: "{shape.height}",
                                    fill: "{shape.color}",
                                    stroke: "black"
                                }
                            },
                            ShapeType::Circle => rsx! {
                                circle {
                                    cx: "{shape.x + shape.width / 2.0}",
                                    cy: "{shape.y + shape.height / 2.0}",
                                    r: "{shape.width / 2.0}",
                                    fill: "{shape.color}",
                                    stroke: "black"
                                }
                            },
                            ShapeType::Line => rsx! {
                                line {
                                    x1: "{shape.x}",
                                    y1: "{shape.y}",
                                    x2: "{shape.x + shape.width}",
                                    y2: "{shape.y + shape.height}",
                                    stroke: "{shape.color}",
                                    stroke_width: "2"
                                }
                            },
                        }
                    }
                }
            }

            // Controls
            div { class: "flex gap-4 mt-4",
                button {
                    class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                    onclick: move |_| {
                        // Placeholder for Play logic
                        println!("Play clicked");
                    },
                    "Play"
                }
                button {
                    class: "px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600",
                    onclick: move |_| {
                        // Placeholder for Wipe logic
                        state.apply_action(crate::model::Action::Wipe);
                    },
                    "Wipe"
                }
            }
            
            div { class: "mt-4 text-sm text-gray-500",
                "History count: {state.history.read().len()}"
            }
        }
    }
}
