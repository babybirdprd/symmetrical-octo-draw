use dioxus::prelude::*;
use crate::model::{AgentConfig, AgentProvider};
use crate::agent::{update_agent_config, get_agent_config};

#[component]
pub fn Settings() -> Element {
    let mut config = use_signal(|| AgentConfig::default());
    let mut is_loading = use_signal(|| true);

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
        spawn(async move {
            if let Err(e) = update_agent_config(current_config).await {
                // In a real app we'd show a toast
                println!("Error saving settings: {}", e);
            } else {
                println!("Settings saved");
            }
        });
    };

    if is_loading() {
        return rsx! { div { "Loading settings..." } };
    }

    rsx! {
        div { class: "p-4 bg-white border rounded shadow-lg w-full max-w-md",
            h2 { class: "text-xl font-bold mb-4", "Agent Settings" }

            // Provider Selection
            div { class: "mb-4",
                label { class: "block text-sm font-medium text-gray-700", "Provider" }
                select {
                    class: "mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50",
                    onchange: move |evt| {
                        let val = evt.value();
                        let provider = match val.as_str() {
                            "Anthropic" => AgentProvider::Anthropic,
                            _ => AgentProvider::OpenAI,
                        };
                        config.write().provider = provider;
                    },
                    option { value: "OpenAI", selected: config.read().provider == AgentProvider::OpenAI, "OpenAI" }
                    option { value: "Anthropic", selected: config.read().provider == AgentProvider::Anthropic, "Anthropic" }
                }
            }

            // Model Name
            div { class: "mb-4",
                label { class: "block text-sm font-medium text-gray-700", "Model" }
                input {
                    class: "mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50",
                    r#type: "text",
                    value: "{config.read().model}",
                    oninput: move |evt| config.write().model = evt.value()
                }
            }

            // API Key
            div { class: "mb-4",
                label { class: "block text-sm font-medium text-gray-700", "API Key" }
                input {
                    class: "mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50",
                    r#type: "password",
                    value: "{config.read().api_key}",
                    oninput: move |evt| config.write().api_key = evt.value()
                }
            }

            // Research Topic
            div { class: "mb-4",
                label { class: "block text-sm font-medium text-gray-700", "Research Topic" }
                input {
                    class: "mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50",
                    r#type: "text",
                    value: "{config.read().research_topic}",
                    oninput: move |evt| config.write().research_topic = evt.value()
                }
            }

            // System Prompt
            div { class: "mb-4",
                label { class: "block text-sm font-medium text-gray-700", "System Prompt" }
                textarea {
                    class: "mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50 h-32",
                    value: "{config.read().system_prompt}",
                    oninput: move |evt| config.write().system_prompt = evt.value()
                }
            }

            // Save Button
            button {
                class: "w-full px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition",
                onclick: save_settings,
                "Save & Restart Agent"
            }
        }
    }
}
