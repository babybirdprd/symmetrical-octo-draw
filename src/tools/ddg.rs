use radkit::tools::{FunctionTool, ToolResult};
use serde::Deserialize;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::json;

#[derive(Deserialize)]
pub struct DuckDuckGoArgs {
    query: String,
}

pub fn make_ddg_tool() -> FunctionTool {
    FunctionTool::new(
        "duckduckgo_search",
        "Search the web using DuckDuckGo to find information about a topic.",
        |args, _ctx| Box::pin(async move {
            // Convert args to Value if it's not already (it is passed as Value by framework usually, but here manual call might pass Value).
            // But if the signature is `Value`, then `from_value` works.
            // The error said `args` is `HashMap<String, Value>`.
            // Wait, why? `FunctionTool` defined in `radkit` likely uses `serde_json::Value` as args?
            // Or maybe `FunctionTool::new` takes a closure where args is `Value`?
            // In `examples/tool_usage.rs`:
            // `|args, _ctx| Box::pin(async move { let a = args["a"]...`
            // implies args is a `Value` (implementing Index).
            // The error `expected Value, found HashMap` implies that in `radkit` version I am using, the closure argument might be `HashMap`?
            // Let's assume it IS `HashMap<String, Value>` based on the error.
            // If so, `serde_json::to_value(args)` -> Value -> `from_value`.

            // However, the error said: `expected enum serde_json::Value found struct HashMap<std::string::String, serde_json::Value>`
            // This means the `args` variable in the closure IS a HashMap.

            let args_value = serde_json::Value::Object(args.into_iter().collect());

            let args: DuckDuckGoArgs = match serde_json::from_value(args_value) {
                Ok(a) => a,
                Err(e) => return ToolResult::error(format!("Invalid arguments: {}", e)),
            };

            let client = Client::new();
            let url = format!("https://html.duckduckgo.com/html/?q={}", args.query);
            
            match client.get(&url).header("User-Agent", "Mozilla/5.0").send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let body = match resp.text().await {
                            Ok(text) => text,
                            Err(e) => return ToolResult::error(format!("Failed to read body: {}", e)),
                        };

                        let document = Html::parse_document(&body);
                        let result_selector = Selector::parse(".result__body").unwrap();
                        
                        let mut results = Vec::new();
                        for element in document.select(&result_selector).take(3) {
                            let title_sel = Selector::parse(".result__title").unwrap();
                            let snippet_sel = Selector::parse(".result__snippet").unwrap();
                            
                            let title = element.select(&title_sel).next().map(|e| e.text().collect::<String>()).unwrap_or_default();
                            let snippet = element.select(&snippet_sel).next().map(|e| e.text().collect::<String>()).unwrap_or_default();
                            
                            if !title.is_empty() {
                                results.push(format!("Title: {}\nSnippet: {}", title.trim(), snippet.trim()));
                            }
                        }
                        
                        let output = if results.is_empty() {
                            "No results found.".to_string()
                        } else {
                            results.join("\n\n")
                        };
                        
                        ToolResult::success(json!({ "result": output }))
                    } else {
                        ToolResult::error(format!("Request failed with status: {}", resp.status()))
                    }
                },
                Err(e) => ToolResult::error(format!("Network error: {}", e))
            }
        })
    )
}
