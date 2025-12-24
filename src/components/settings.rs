use crate::agent::{get_agent_config, update_agent_config};
use crate::model::{AgentConfig, AgentProvider};
use dioxus::prelude::*;

#[component]
pub fn Settings(on_close: EventHandler<()>) -> Element {
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
                save_status.set("Saved!".to_string());
            }
        });
    };

    if is_loading() {
        return rsx! { div { class: "p-6 text-gray-400", "Loading..." } };
    }

    let current_provider = config.read().provider.clone();

    rsx! {
        div { class: "p-6 space-y-4",
            div { class: "flex justify-between items-center mb-4",
                h2 { class: "text-xl font-bold text-white", "API Settings" }
                button {
                    class: "text-gray-400 hover:text-white text-2xl",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            // Provider
            div {
                label { class: "block text-sm font-medium text-gray-300 mb-1", "Provider" }
                select {
                    class: "w-full p-3 bg-gray-700 border border-gray-600 rounded-lg text-white focus:ring-2 focus:ring-blue-500",
                    onchange: move |evt| {
                        let val = evt.value();
                        let provider = match val.as_str() {
                            "Anthropic" => AgentProvider::Anthropic,
                            "OpenRouter" => AgentProvider::OpenRouter,
                            "Google Gemini" => AgentProvider::Gemini,
                            "Grok" => AgentProvider::Grok,
                            "DeepSeek" => AgentProvider::DeepSeek,
                            _ => AgentProvider::OpenAI,
                        };
                        let mut cfg = config.write();
                        cfg.provider = provider.clone();
                        if cfg.model.is_empty() {
                            cfg.model = provider.default_model().to_string();
                        }
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

            // Model
            div {
                label { class: "block text-sm font-medium text-gray-300 mb-1", "Model" }
                input {
                    class: "w-full p-3 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:ring-2 focus:ring-blue-500",
                    r#type: "text",
                    placeholder: current_provider.default_model(),
                    value: "{config.read().model}",
                    oninput: move |evt| config.write().model = evt.value()
                }
            }

            // API Key
            div {
                label { class: "block text-sm font-medium text-gray-300 mb-1", "API Key" }
                input {
                    class: "w-full p-3 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:ring-2 focus:ring-blue-500",
                    r#type: "password",
                    placeholder: "Enter your API key",
                    value: "{config.read().api_key}",
                    oninput: move |evt| config.write().api_key = evt.value()
                }
            }

            // System Prompt
            div {
                label { class: "block text-sm font-medium text-gray-300 mb-1", "System Prompt" }
                textarea {
                    class: "w-full p-3 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:ring-2 focus:ring-blue-500 h-20 resize-none",
                    value: "{config.read().system_prompt}",
                    oninput: move |evt| config.write().system_prompt = evt.value()
                }
            }

            // Save
            button {
                class: "w-full py-3 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-semibold transition-colors",
                onclick: save_settings,
                "Save"
            }

            if !save_status().is_empty() {
                p {
                    class: if save_status().starts_with("Error") { "text-red-400 text-sm" } else { "text-green-400 text-sm" },
                    "{save_status()}"
                }
            }
        }
    }
}
