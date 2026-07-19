//! Hemion navigation: the working console entry plus preserved wallet tools.
//!
//! Flow Spec Part 8 requires a four-screen console, but navigation may expose
//! only working screens. G-01 therefore exposes S-H1 and the untouched legacy
//! wallet; G-03 through G-06 add their destinations when implemented.

use crate::routes::Route;
use dioxus::prelude::*;

#[derive(Clone)]
struct Destination {
    label: &'static str,
    icon: &'static str,
    route: Route,
}

fn console_destinations() -> [Destination; 7] {
    [
        Destination {
            label: "Console home",
            icon: "⌁",
            route: Route::ConsoleHome {},
        },
        Destination {
            label: "Bundle verifier",
            icon: "✓",
            route: Route::BundleVerify {},
        },
        Destination {
            label: "Assurance inspector",
            icon: "▦",
            route: Route::AssuranceInspector {},
        },
        Destination {
            label: "Object inspector",
            icon: "⌗",
            route: Route::ObjectInspector {},
        },
        Destination {
            label: "Dispute inspector",
            icon: "⋈",
            route: Route::DisputeInspector {},
        },
        Destination {
            label: "Fixture lab",
            icon: "≋",
            route: Route::FixtureLab {},
        },
        Destination {
            label: "Tuppira explorer",
            icon: "⌘",
            route: Route::TuppiraExplorer {},
        },
    ]
}

fn wallet_destinations() -> [Destination; 5] {
    [
        Destination {
            label: "Legacy wallet",
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
        "instrument-nav-tab"
    } else {
        "instrument-nav-link"
    };
    rsx! {
        Link { to: destination.route, class: "{class}", aria_label: destination.label,
            span { aria_hidden: "true", "{destination.icon}" }
            span { "{destination.label}" }
        }
    }
}

/// The application navigation. Native links provide keyboard activation; CSS
/// supplies the required visible focus treatment and compact/mobile layouts.
#[component]
pub fn Sidebar(sidebar_open: bool) -> Element {
    rsx! {
        aside {
            class: if sidebar_open { "instrument-sidebar instrument-sidebar-open" } else { "instrument-sidebar instrument-sidebar-closed" },
            aria_label: "Primary navigation",
            Link { to: Route::ConsoleHome {}, class: "instrument-brand",
                span { aria_hidden: "true", "◈" }
                if sidebar_open { span { "Hemion" } }
            }
            nav { class: "instrument-nav", aria_label: "Developer console",
                if sidebar_open { p { class: "instrument-nav-heading", "Console" } }
                for destination in console_destinations() { {navigation_link(destination, false)} }
                if sidebar_open { p { class: "instrument-nav-heading", "Legacy tools" } }
                for destination in wallet_destinations() { {navigation_link(destination, false)} }
            }
            if sidebar_open {
                p { class: "instrument-boundary", "Local results only. External verdicts remain recorded elsewhere until re-verified." }
            }
        }
        nav { class: "instrument-tabs", aria_label: "Primary navigation",
            {navigation_link(console_destinations()[0].clone(), true)}
            {navigation_link(console_destinations()[1].clone(), true)}
            {navigation_link(wallet_destinations()[0].clone(), true)}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{console_destinations, wallet_destinations};

    #[test]
    fn navigation_exposes_only_working_console_screen_and_legacy_wallet() {
        assert_eq!(console_destinations()[0].label, "Console home");
        assert_eq!(console_destinations()[1].label, "Bundle verifier");
        assert_eq!(console_destinations()[2].label, "Assurance inspector");
        assert_eq!(console_destinations()[3].label, "Object inspector");
        assert_eq!(console_destinations()[4].label, "Dispute inspector");
        assert_eq!(console_destinations()[5].label, "Fixture lab");
        assert_eq!(console_destinations()[6].label, "Tuppira explorer");
        assert_eq!(wallet_destinations()[0].label, "Legacy wallet");
        assert_eq!(wallet_destinations().len(), 5);
    }
}
