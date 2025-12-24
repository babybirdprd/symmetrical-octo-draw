use radkit::tools::{FunctionTool, ToolResult};
use serde::Deserialize;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::json;

#[derive(Deserialize)]
pub struct DuckDuckGoArgs {
    /// The search query to look up
    query: String,
}

pub fn make_ddg_tool() -> FunctionTool {
    FunctionTool::new(
        "web_search",
        "Search the web using DuckDuckGo to find information about any topic. Returns titles and snippets from search results.",
        |args, _ctx| Box::pin(async move {
            let args_value = serde_json::Value::Object(args.into_iter().collect());
            
            let args: DuckDuckGoArgs = match serde_json::from_value(args_value) {
                Ok(a) => a,
                Err(e) => return ToolResult::error(format!("Invalid arguments: {}", e)),
            };

            println!("Tool: Searching for '{}'", args.query);

            let client = Client::new();
            let encoded_query = urlencoding::encode(&args.query);
            let url = format!("https://html.duckduckgo.com/html/?q={}", encoded_query);
            
            match client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .send()
                .await 
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let body = match resp.text().await {
                            Ok(text) => text,
                            Err(e) => return ToolResult::error(format!("Failed to read body: {}", e)),
                        };

                        let document = Html::parse_document(&body);
                        
                        // Try multiple selectors for different DDG layouts
                        let result_selectors = [
                            ".result__body",
                            ".result",
                            ".web-result",
                        ];
                        
                        let mut results = Vec::new();
                        
                        for selector_str in result_selectors {
                            if let Ok(result_selector) = Selector::parse(selector_str) {
                                for element in document.select(&result_selector).take(5) {
                                    // Try to get title
                                    let title = if let Ok(title_sel) = Selector::parse(".result__title, .result__a, a") {
                                        element.select(&title_sel)
                                            .next()
                                            .map(|e| e.text().collect::<String>())
                                            .unwrap_or_default()
                                    } else {
                                        String::new()
                                    };
                                    
                                    // Try to get snippet
                                    let snippet = if let Ok(snippet_sel) = Selector::parse(".result__snippet, .result-snippet") {
                                        element.select(&snippet_sel)
                                            .next()
                                            .map(|e| e.text().collect::<String>())
                                            .unwrap_or_default()
                                    } else {
                                        String::new()
                                    };
                                    
                                    let title = title.trim().to_string();
                                    let snippet = snippet.trim().to_string();
                                    
                                    if !title.is_empty() || !snippet.is_empty() {
                                        results.push(json!({
                                            "title": title,
                                            "snippet": snippet
                                        }));
                                    }
                                }
                                
                                if !results.is_empty() {
                                    break;
                                }
                            }
                        }
                        
                        // Fallback: extract any text
                        if results.is_empty() {
                            let text: String = document.root_element()
                                .text()
                                .collect::<Vec<_>>()
                                .join(" ")
                                .chars()
                                .take(1000)
                                .collect();
                            results.push(json!({
                                "title": "Raw content",
                                "snippet": text.trim()
                            }));
                        }
                        
                        let output_text = results.iter()
                            .filter_map(|r| {
                                let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("");
                                let snippet = r.get("snippet").and_then(|v| v.as_str()).unwrap_or("");
                                if title.is_empty() && snippet.is_empty() {
                                    None
                                } else {
                                    Some(format!("📄 {}\n{}", title, snippet))
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n\n");
                        
                        println!("Tool: Search found {} results", results.len());
                        
                        ToolResult::success(json!({ 
                            "query": args.query,
                            "result_count": results.len(),
                            "results": output_text
                        }))
                    } else {
                        ToolResult::error(format!("Request failed with status: {}", resp.status()))
                    }
                },
                Err(e) => ToolResult::error(format!("Network error: {}", e))
            }
        })
    )
}
