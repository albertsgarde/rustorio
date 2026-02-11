mod backend;

use backend::{get_gamemodes, get_leaderboard};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/?:gamemode")]
    Home { gamemode: String },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(feature = "server")]
    backend::server::init();

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

/// Home page
#[component]
fn Home(gamemode: String) -> Element {
    let nav = use_navigator();
    let gamemodes = use_server_future(get_gamemodes)?;
    let selected_gamemode = if gamemode.is_empty() {
        "tutorial".to_string()
    } else {
        gamemode
    };

    let cloned_gamemode = selected_gamemode.clone();
    let entries = use_server_future(move || {
        let gamemode = cloned_gamemode.clone();
        async move { get_leaderboard(gamemode).await }
    })?;

    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-2xl font-bold mb-4", "Leaderboard" }
            match gamemodes() {
                Some(Ok(modes)) => rsx! {
                    select {
                        class: "mb-4 p-2 border rounded",
                        onchange: move |e: Event<FormData>| {
                            nav.push(Route::Home { gamemode: e.value() });
                        },
                        for mode in modes {
                            option { value: "{mode}", selected: selected_gamemode == mode, "{mode}" }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    p { class: "text-red-500", "Error loading gamemodes: {e}" }
                },
                None => rsx! {
                    p { "Loading gamemodes..." }
                },
            }
            match entries() {
                Some(Ok(entries)) => rsx! {
                    table { class: "w-full border-collapse",
                        thead {
                            tr { class: "border-b",
                                th { class: "text-left p-2", "Player" }
                                th { class: "text-right p-2", "Ticks" }
                            }
                        }
                        tbody {
                            for (i , entry) in entries.iter().enumerate() {
                                tr { class: "border-b hover:bg-gray-100",
                                    td { class: "p-2", "{i + 1}. {entry.name}" }
                                    td { class: "text-right p-2", "{entry.ticks}" }
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    p { class: "text-red-500", "Error loading leaderboard: {e}" }
                },
                None => rsx! {
                    p { "Loading..." }
                },
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        nav { class: "flex justify-end items-center py-4 pr-4 gap-4",
            a {
                class: "flex items-center p-2 rounded-md text-white hover:text-slate-300 hover:bg-white/10 transition-colors",
                href: "https://github.com/albertsgarde/rustorio",
                target: "_blank",
                rel: "noopener noreferrer",
                "aria-label": "GitHub",
                svg {
                    class: "fill-current",
                    width: "24",
                    height: "24",
                    view_box: "0 0 24 24",
                    path { d: "M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" }
                }
            }
            a {
                class: "flex items-center p-2 rounded-md text-white hover:text-slate-300 hover:bg-white/10 transition-colors",
                href: "https://discord.gg/uKJugp85Fk",
                target: "_blank",
                rel: "noopener noreferrer",
                "aria-label": "Discord",
                svg {
                    class: "fill-current",
                    width: "24",
                    height: "24",
                    view_box: "0 0 24 24",
                    path {
                        d: "M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z",
                    }
                }
            }
        }

        Outlet::<Route> {}
    }
}
