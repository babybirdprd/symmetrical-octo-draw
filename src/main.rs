use dioxus::prelude::*;

mod agent;
mod components;
mod model;
mod state;
mod tools;
mod views;

#[cfg(feature = "server")]
mod server_state;

// Simple single-page app - no router needed
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::thread;
        thread::spawn(|| {
            agent::init_agent();
        });
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        views::Home {}
    }
}
