//! One adaptive, task-oriented navigation for the wallet.
//!
//! The presentation changes with the viewport, but the information
//! architecture does not: the same five destinations are rendered as a wide
//! sidebar and a narrow, touch-sized bottom tab bar.  There is intentionally
//! no persona or protocol-role switcher here.

use crate::routes::Route;
use dioxus::prelude::*;

#[derive(Clone)]
struct Destination {
    label: &'static str,
    icon: &'static str,
    route: Route,
}

fn destinations() -> [Destination; 5] {
    [
        Destination {
            label: "Home",
            icon: "⌂",
            route: Route::Dashboard {},
        },
        Destination {
            label: "Assets",
            icon: "◇",
            route: Route::Assets {},
        },
        Destination {
            label: "Activity",
            icon: "↔",
            route: Route::Activity {},
        },
        Destination {
            label: "Contacts",
            icon: "♙",
            route: Route::Contacts {},
        },
        Destination {
            label: "Settings",
            icon: "⚙",
            route: Route::Settings {},
        },
    ]
}

fn navigation_link(destination: Destination, compact: bool) -> Element {
    let class = if compact {
        "wallet-nav-tab min-h-11 min-w-11 flex flex-1 flex-col items-center justify-center gap-0.5 rounded-lg px-2 py-1 text-xs text-gray-300 hover:bg-gray-800 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-400"
    } else {
        "wallet-nav-link min-h-11 flex items-center gap-3 rounded-lg px-3 py-2 text-sm text-gray-300 hover:bg-gray-800 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-400"
    };

    rsx! {
        Link { to: destination.route, class: "{class}", aria_label: destination.label,
            span { class: "text-base", aria_hidden: "true", "{destination.icon}" }
            span { "{destination.label}" }
        }
    }
}

/// The only application navigation component. CSS breakpoints select its
/// sidebar or tab-bar presentation; page code never selects a platform UI.
#[component]
pub fn Sidebar(sidebar_open: bool) -> Element {
    let destinations = destinations();
    let sidebar_destinations = destinations.clone();

    rsx! {
        aside {
            class: if sidebar_open {
                "wallet-nav-sidebar app-sidebar w-64 bg-gray-900 border-r border-gray-800 flex-shrink-0 flex flex-col h-screen sticky top-0"
            } else {
                "wallet-nav-sidebar app-sidebar w-16 bg-gray-900 border-r border-gray-800 flex-shrink-0 flex flex-col h-screen sticky top-0"
            },
            aria_label: "Primary navigation",
            div { class: "min-h-16 border-b border-gray-800 px-4 py-4",
                Link { to: Route::Dashboard {}, class: "flex min-h-11 items-center gap-2 rounded-lg focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-400",
                    span { aria_hidden: "true", "◈" }
                    if sidebar_open {
                        span { class: "text-lg font-bold text-gray-100", "CSV Wallet" }
                    }
                }
            }
            nav { class: "flex flex-1 flex-col gap-1 p-2",
                for destination in sidebar_destinations {
                    {navigation_link(destination, false)}
                }
            }
            if sidebar_open {
                p { class: "px-4 pb-4 text-xs text-gray-500", "Protocol tools are available contextually or in Settings → Advanced." }
            }
        }

        nav { class: "wallet-nav-tabs fixed inset-x-0 bottom-0 z-40 border-t border-gray-800 bg-gray-900/95 px-2 py-1 backdrop-blur", aria_label: "Primary navigation",
            div { class: "mx-auto flex max-w-xl gap-1",
                for destination in destinations {
                    {navigation_link(destination, true)}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::destinations;

    #[test]
    fn navigation_has_exactly_the_five_task_destinations() {
        let labels: Vec<_> = destinations().into_iter().map(|item| item.label).collect();
        assert_eq!(
            labels,
            ["Home", "Assets", "Activity", "Contacts", "Settings"]
        );
    }
}
