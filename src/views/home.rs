use crate::agent::{get_agent_config, poll_agent, update_agent_config};
use crate::components::settings::Settings;
use crate::model::{Action, ShapeType};
use crate::state::BoardState;
use dioxus::prelude::*;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use gloo_timers::future::sleep;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;

#[component]
pub fn Home() -> Element {
    let mut state = use_context_provider(|| BoardState::new());
    let mut show_settings = use_signal(|| false);
    let mut research_input = use_signal(|| String::new());
    let mut is_researching = use_signal(|| false);
    let mut last_index = use_signal(|| 0);
    let mut playback_active = use_signal(|| false);

    // Polling Loop for agent actions
    use_coroutine(move |mut _rx: UnboundedReceiver<()>| async move {
        sleep(Duration::from_millis(500)).await;
        loop {
            let current_idx = *last_index.read();
            if let Ok(actions) = poll_agent(current_idx).await {
                if !actions.is_empty() {
                    let mut s = state;
                    for action in &actions {
                        s.apply_action(action.clone());
                    }
                    *last_index.write() += actions.len();
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
    });

    // Setup Video Recorder (WASM only)
    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        let _ = document::eval(
            r#"
            window.setupRecorder = () => {
                const canvas = document.createElement('canvas');
                canvas.width = 800;
                canvas.height = 500;
                const ctx = canvas.getContext('2d');
                window._recorder = { canvas, ctx, frames: [] };
            };
            window.startRecording = () => {
                if (!window._recorder) window.setupRecorder();
                window._recorder.frames = [];
            };
            window.captureFrame = (svgId) => {
                const svg = document.getElementById(svgId);
                if (!svg || !window._recorder) return;
                const data = new XMLSerializer().serializeToString(svg);
                const img = new Image();
                img.onload = () => {
                    window._recorder.ctx.clearRect(0, 0, 800, 500);
                    window._recorder.ctx.drawImage(img, 0, 0);
                    window._recorder.frames.push(window._recorder.canvas.toDataURL('image/png'));
                };
                img.src = 'data:image/svg+xml;base64,' + btoa(unescape(encodeURIComponent(data)));
            };
            window.stopRecording = () => {
                if (!window._recorder || window._recorder.frames.length === 0) return;
                // Create animated GIF or download frames
                const link = document.createElement('a');
                link.download = 'presentation.png';
                link.href = window._recorder.frames[window._recorder.frames.length - 1];
                link.click();
            };
            window.setupRecorder();
        "#,
        );
    });

    rsx! {
        div { class: "min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 text-white",
            // Header
            header { class: "flex items-center justify-between px-6 py-4 border-b border-gray-700/50",
                h1 { class: "text-2xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent",
                    "Agent Excalidraw"
                }
                button {
                    class: "p-2 rounded-lg bg-gray-800 hover:bg-gray-700 transition-colors text-xl",
                    onclick: move |_| show_settings.set(!show_settings()),
                    "⚙️"
                }
            }

            // Settings Modal
            if show_settings() {
                div { class: "fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center",
                    onclick: move |_| show_settings.set(false),
                    div {
                        class: "bg-gray-800 rounded-xl shadow-2xl max-w-md w-full mx-4",
                        onclick: move |e| e.stop_propagation(),
                        Settings { on_close: move |_| show_settings.set(false) }
                    }
                }
            }

            // Main Content
            main { class: "container mx-auto px-6 py-8",
                // Research Input
                div { class: "max-w-3xl mx-auto mb-8",
                    div { class: "flex gap-3",
                        input {
                            class: "flex-1 px-5 py-4 bg-gray-800/50 border border-gray-700 rounded-xl text-lg placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all",
                            r#type: "text",
                            placeholder: "What would you like to research?",
                            value: "{research_input}",
                            oninput: move |e| research_input.set(e.value()),
                            onkeypress: move |e| {
                                if e.key() == Key::Enter {
                                    let topic = research_input.read().clone();
                                    if !topic.is_empty() {
                                        is_researching.set(true);
                                        spawn(async move {
                                            if let Ok(mut config) = get_agent_config().await {
                                                config.research_topic = topic;
                                                let _ = update_agent_config(config).await;
                                            }
                                        });
                                    }
                                }
                            }
                        }
                        button {
                            class: "px-8 py-4 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-500 hover:to-purple-500 rounded-xl font-semibold transition-all transform hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed",
                            disabled: research_input.read().is_empty(),
                            onclick: move |_| {
                                let topic = research_input.read().clone();
                                if !topic.is_empty() {
                                    is_researching.set(true);
                                    spawn(async move {
                                        if let Ok(mut config) = get_agent_config().await {
                                            config.research_topic = topic;
                                            let _ = update_agent_config(config).await;
                                        }
                                    });
                                }
                            },
                            if is_researching() { "Researching..." } else { "Go →" }
                        }
                    }
                }

                // Canvas
                div { class: "max-w-4xl mx-auto",
                    div { class: "bg-white rounded-2xl shadow-2xl overflow-hidden",
                        svg {
                            id: "board-svg",
                            class: "w-full",
                            view_box: "0 0 800 500",
                            style: "background: linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%);",
                            for shape in state.board.read().shapes.iter() {
                                match shape.shape_type {
                                    ShapeType::Rectangle => rsx! {
                                        rect {
                                            x: "{shape.x}",
                                            y: "{shape.y}",
                                            width: "{shape.width}",
                                            height: "{shape.height}",
                                            fill: "{shape.color}",
                                            stroke: "#333",
                                            stroke_width: "2",
                                            rx: "8"
                                        }
                                    },
                                    ShapeType::Circle => rsx! {
                                        circle {
                                            cx: "{shape.x + shape.width / 2.0}",
                                            cy: "{shape.y + shape.height / 2.0}",
                                            r: "{shape.width / 2.0}",
                                            fill: "{shape.color}",
                                            stroke: "#333",
                                            stroke_width: "2"
                                        }
                                    },
                                    ShapeType::Line => rsx! {
                                        line {
                                            x1: "{shape.x}",
                                            y1: "{shape.y}",
                                            x2: "{shape.x + shape.width}",
                                            y2: "{shape.y + shape.height}",
                                            stroke: "{shape.color}",
                                            stroke_width: "3",
                                            stroke_linecap: "round"
                                        }
                                    },
                                }
                            }
                        }
                    }
                }

                // Controls
                div { class: "flex justify-center gap-4 mt-6",
                    button {
                        class: "px-6 py-3 bg-gray-700 hover:bg-gray-600 rounded-xl font-medium transition-colors flex items-center gap-2",
                        disabled: playback_active(),
                        onclick: move |_| {
                            playback_active.set(true);
                            let mut s = state;
                            let history = s.history.read().clone();
                            s.board.write().shapes.clear();
                            spawn(async move {
                                for action in history {
                                    sleep(Duration::from_millis(400)).await;
                                    match action {
                                        Action::Draw(shape) => { s.board.write().shapes.push(shape); }
                                        Action::Wipe => { s.board.write().shapes.clear(); }
                                        Action::NewBoard => { s.board.write().shapes.clear(); }
                                    }
                                }
                                playback_active.set(false);
                            });
                        },
                        "▶ Play"
                    }
                    button {
                        class: "px-6 py-3 bg-gray-700 hover:bg-red-600 rounded-xl font-medium transition-colors",
                        onclick: move |_| {
                            state.apply_action(Action::Wipe);
                        },
                        "🗑 Clear"
                    }
                    button {
                        class: "px-6 py-3 bg-gray-700 hover:bg-green-600 rounded-xl font-medium transition-colors",
                        onclick: move |_| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                playback_active.set(true);
                                let mut s = state;
                                let history = s.history.read().clone();
                                s.board.write().shapes.clear();
                                spawn(async move {
                                    let _ = document::eval("window.startRecording()");
                                    let _ = document::eval("window.captureFrame('board-svg')");
                                    for action in history {
                                        sleep(Duration::from_millis(400)).await;
                                        match action {
                                            Action::Draw(shape) => { s.board.write().shapes.push(shape); }
                                            Action::Wipe => { s.board.write().shapes.clear(); }
                                            Action::NewBoard => { s.board.write().shapes.clear(); }
                                        }
                                        sleep(Duration::from_millis(50)).await;
                                        let _ = document::eval("window.captureFrame('board-svg')");
                                    }
                                    sleep(Duration::from_millis(500)).await;
                                    let _ = document::eval("window.stopRecording()");
                                    playback_active.set(false);
                                });
                            }
                        },
                        "📹 Export"
                    }
                }

                // Status
                div { class: "text-center mt-6 text-gray-500 text-sm",
                    "Shapes: {state.board.read().shapes.len()} | History: {state.history.read().len()}"
                }
            }
        }
    }
}
