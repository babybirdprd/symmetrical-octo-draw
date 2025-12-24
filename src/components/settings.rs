use crate::agent::{get_agent_config, update_agent_config};
use crate::model::{AgentConfig, AgentProvider};
use dioxus::prelude::*;

#[component]
pub fn Settings() -> Element {
    let mut config = use_signal(|| AgentConfig::default());
    let mut is_loading = use_signal(|| true);
    let mut save_status = use_signal(|| String::new());

    // Fetch initial config
    use_effect(move || {
        spawn(async move {
            if let Ok(server_config) = get_agent_config().await {
                config.set(server_config);
            }
            is_loading.set(false);
        });
    });

    let save_settings = move |_| {
        let current_config = config.read().clone();
        save_status.set("Saving...".to_string());
        spawn(async move {
            if let Err(e) = update_agent_config(current_config).await {
                save_status.set(format!("Error: {}", e));
            } else {
                save_status.set("Saved! Agent will use new config.".to_string());
            }
        });
    };

    if is_loading() {
        return rsx! { div { class: "p-4", "Loading settings..." } };
    }

    let current_provider = config.read().provider.clone();
    let show_base_url = current_provider.supports_base_url();

    rsx! {
        div { class: "p-4 bg-white border rounded shadow-lg w-full max-w-md space-y-4",
            h2 { class: "text-xl font-bold mb-2", "Agent Settings" }

            // Provider Selection
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "Provider" }
                select {
                    class: "w-full p-2 border rounded focus:ring-2 focus:ring-blue-500",
                    value: "{current_provider.display_name()}",
                    onchange: move |evt| {
                        let val = evt.value();
                        let provider = match val.as_str() {
                            "Anthropic" => AgentProvider::Anthropic,
                            "Google Gemini" => AgentProvider::Gemini,
                            "Groq" => AgentProvider::Groq,
                            "OpenRouter" => AgentProvider::OpenRouter,
                            "Custom (OpenAI-compatible)" => AgentProvider::Custom,
                            _ => AgentProvider::OpenAI,
                        };
                        // Update provider and set default model if empty
                        let mut cfg = config.write();
                        if cfg.model.is_empty() {
                            cfg.model = provider.default_model().to_string();
                        }
                        cfg.provider = provider;
                    },
                    for provider in AgentProvider::all() {
                        option {
                            value: "{provider.display_name()}",
                            selected: config.read().provider == provider,
                            "{provider.display_name()}"
                        }
                    }
                }
            }

            // Model Name
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "Model Name" }
                input {
                    class: "w-full p-2 border rounded focus:ring-2 focus:ring-blue-500",
                    r#type: "text",
                    placeholder: "e.g. gpt-4o, claude-3-5-sonnet-20241022",
                    value: "{config.read().model}",
                    oninput: move |evt| config.write().model = evt.value()
                }
                p { class: "text-xs text-gray-500 mt-1",
                    "Enter the exact model name from your provider"
                }
            }

            // API Key
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "API Key" }
                input {
                    class: "w-full p-2 border rounded focus:ring-2 focus:ring-blue-500",
                    r#type: "password",
                    placeholder: "sk-...",
                    value: "{config.read().api_key}",
                    oninput: move |evt| config.write().api_key = evt.value()
                }
            }

            // Base URL (only shown for providers that support it)
            if show_base_url {
                div {
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Base URL (optional)" }
                    input {
                        class: "w-full p-2 border rounded focus:ring-2 focus:ring-blue-500",
                        r#type: "text",
                        placeholder: "https://api.openai.com/v1",
                        value: "{config.read().base_url.clone().unwrap_or_default()}",
                        oninput: move |evt| {
                            let val = evt.value();
                            config.write().base_url = if val.is_empty() { None } else { Some(val) };
                        }
                    }
                    p { class: "text-xs text-gray-500 mt-1",
                        "For Azure, local LLMs, or custom endpoints"
                    }
                }
            }

            // Research Topic
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "Research Topic" }
                input {
                    class: "w-full p-2 border rounded focus:ring-2 focus:ring-blue-500",
                    r#type: "text",
                    placeholder: "What should the agent research?",
                    value: "{config.read().research_topic}",
                    oninput: move |evt| config.write().research_topic = evt.value()
                }
            }

            // System Prompt
            div {
                label { class: "block text-sm font-medium text-gray-700 mb-1", "System Prompt" }
                textarea {
                    class: "w-full p-2 border rounded focus:ring-2 focus:ring-blue-500 h-24",
                    value: "{config.read().system_prompt}",
                    oninput: move |evt| config.write().system_prompt = evt.value()
                }
            }

            // Save Button
            button {
                class: "w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition font-medium",
                onclick: save_settings,
                "Save & Apply"
            }

            // Status Message
            if !save_status().is_empty() {
                p {
                    class: if save_status().starts_with("Error") { "text-red-600 text-sm" } else { "text-green-600 text-sm" },
                    "{save_status()}"
                }
            }
        }
    }
}
