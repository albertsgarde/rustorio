use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

/// Represents a player's score on the leaderboard
#[derive(Clone, PartialEq)]
struct LeaderboardEntry {
    name: String,
    ticks: u64,
}

/// Home page
#[component]
fn Home() -> Element {
    let entries = vec![
        LeaderboardEntry { name: "SpeedRunner42".to_string(), ticks: 12_543 },
        LeaderboardEntry { name: "FactorioMaster".to_string(), ticks: 15_221 },
        LeaderboardEntry { name: "OptimalPath".to_string(), ticks: 18_902 },
        LeaderboardEntry { name: "RocketScience".to_string(), ticks: 21_445 },
        LeaderboardEntry { name: "NewPlayer".to_string(), ticks: 45_678 },
    ];

    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-2xl font-bold mb-4", "Leaderboard" }
            table { class: "w-full border-collapse",
                thead {
                    tr { class: "border-b",
                        th { class: "text-left p-2", "Player" }
                        th { class: "text-right p-2", "Ticks" }
                    }
                }
                tbody {
                    for (i, entry) in entries.iter().enumerate() {
                        tr { class: "border-b hover:bg-gray-100",
                            td { class: "p-2", "{i + 1}. {entry.name}" }
                            td { class: "text-right p-2", "{entry.ticks}" }
                        }
                    }
                }
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar" }

        Outlet::<Route> {}
    }
}
