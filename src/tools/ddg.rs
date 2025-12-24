use radkit::tools::{FunctionTool, ToolResult};
use serde::Deserialize;
use reqwest::blocking::Client;
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
            // Parse args manually
            let args: DuckDuckGoArgs = match serde_json::from_value(args) {
                Ok(a) => a,
                Err(e) => return ToolResult::error(format!("Invalid arguments: {}", e)),
            };

            let client = Client::new();
            let url = format!("https://html.duckduckgo.com/html/?q={}", args.query);
            
            // Blocking call inside async block - use spawn_blocking in real app
            match client.get(&url).header("User-Agent", "Mozilla/5.0").send() {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let body = resp.text().unwrap_or_default();
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
