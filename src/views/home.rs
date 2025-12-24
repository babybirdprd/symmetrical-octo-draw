use crate::agent::poll_agent;
use crate::components::settings::Settings;
use crate::model::{Action, ShapeType};
use crate::state::BoardState;
use dioxus::prelude::*;
use std::time::Duration;

// WASM-compatible async sleep
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::sleep;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    let mut state = use_context_provider(|| BoardState::new());
    let mut show_settings = use_signal(|| false);

    // Track last received index for reliable polling
    let mut last_index = use_signal(|| 0);

    // Polling Loop
    use_coroutine(move |mut _rx: UnboundedReceiver<()>| async move {
        // Wait for hydration/mount
        sleep(Duration::from_millis(100)).await;

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

    // Playback state
    let mut playback_active = use_signal(|| false);

    // Video export JS helper - uses document::eval() in WASM

    // Setup Recorder
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = document::eval(
                r#"
                window.setupRecorder = () => {
                    const canvas = document.createElement('canvas');
                    canvas.width = 800;
                    canvas.height = 600;
                    window.exportCanvas = canvas;
                    window.exportCtx = canvas.getContext('2d');

                    const stream = canvas.captureStream(30);
                    window.recorder = new MediaRecorder(stream, { mimeType: 'video/webm' });
                    window.chunks = [];

                    window.recorder.ondataavailable = (e) => window.chunks.push(e.data);
                    window.recorder.onstop = () => {
                        const blob = new Blob(window.chunks, { type: 'video/webm' });
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = 'presentation.webm';
                        a.click();
                        URL.revokeObjectURL(url);
                        window.chunks = [];
                    };
                };

                window.captureFrame = (svgId) => {
                    return new Promise((resolve) => {
                        const svg = document.getElementById(svgId);
                        if (!svg) { resolve(); return; }

                        const svgData = new XMLSerializer().serializeToString(svg);
                        const img = new Image();
                        const svgBlob = new Blob([svgData], {type: 'image/svg+xml;charset=utf-8'});
                        const url = URL.createObjectURL(svgBlob);

                        img.onload = () => {
                            window.exportCtx.fillStyle = 'white';
                            window.exportCtx.fillRect(0, 0, 800, 600);
                            window.exportCtx.drawImage(img, 0, 0);
                            URL.revokeObjectURL(url);
                            resolve();
                        };
                        img.src = url;
                    });
                };

                window.startRecording = () => {
                    if (!window.recorder) window.setupRecorder();
                    window.chunks = [];
                    window.recorder.start();
                };

                window.stopRecording = () => {
                    if (window.recorder && window.recorder.state !== 'inactive') {
                        window.recorder.stop();
                    }
                };

                window.setupRecorder();
            "#,
            );
        }
    });

    rsx! {
        div { class: "flex flex-col items-center justify-center min-h-screen bg-gray-100 relative",
            div { class: "absolute top-4 right-4 z-50",
                button {
                    class: "p-2 bg-gray-200 rounded hover:bg-gray-300",
                    onclick: move |_| show_settings.set(!show_settings()),
                    if show_settings() { "Close Settings" } else { "Settings" }
                }
            }

            if show_settings() {
                 div { class: "absolute top-16 right-4 z-50",
                     Settings {}
                 }
            }

            h1 { class: "text-2xl font-bold mb-4", "Agent Excalidraw" }

            // Canvas Area
            div {
                class: "border-2 border-gray-300 bg-white shadow-lg w-[800px] h-[600px] relative",
                id: "board-container",
                svg {
                    id: "board-svg",
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
                    class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50",
                    disabled: playback_active(),
                    onclick: move |_| {
                        playback_active.set(true);
                        let mut s = state; // Copy state
                        let history = s.history.read().clone();
                        s.board.write().shapes.clear();

                        spawn(async move {
                            for action in history {
                                sleep(Duration::from_millis(500)).await;
                                match action {
                                    Action::Draw(shape) => { s.board.write().shapes.push(shape); }
                                    Action::Wipe => { s.board.write().shapes.clear(); }
                                    Action::NewBoard => { s.board.write().shapes.clear(); }
                                }
                            }
                            playback_active.set(false);
                        });
                    },
                    if playback_active() { "Playing..." } else { "Play History" }
                }
                button {
                    class: "px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600",
                    onclick: move |_| {
                        state.apply_action(crate::model::Action::Wipe);
                    },
                    "Wipe"
                }
                button {
                    class: "px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600",
                    onclick: move |_| {
                        // Export Video: Playback + Record
                        #[cfg(target_arch = "wasm32")]
                        {
                            playback_active.set(true);
                            let mut s = state;
                            let history = s.history.read().clone();
                            s.board.write().shapes.clear();

                            // Use document::eval for JS interop

                            spawn(async move {
                                let _ = document::eval("window.startRecording()");
                                let _ = document::eval("window.captureFrame('board-svg')");

                                for action in history {
                                    sleep(Duration::from_millis(500)).await;

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
                        #[cfg(not(target_arch = "wasm32"))]
                        println!("Video export only available in browser");
                    },
                    "Export Video"
                }
            }

            div { class: "mt-4 text-sm text-gray-500",
                "History count: {state.history.read().len()}"
            }

            div { class: "mt-2 text-xs text-gray-400",
                "Agent is running in background (Server side). Refresh to see new agent actions if disconnected."
            }
        }
    }
}
