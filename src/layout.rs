//! Application layout component.

use crate::components::{Header, Sidebar};
use crate::routes::Route;
use dioxus::prelude::*;

/// Main application layout component.
#[component]
pub fn Layout() -> Element {
    let mut sidebar_open = use_signal(|| true);

    rsx! {
        div { class: "app-shell min-h-screen bg-gray-950 text-gray-100 flex",
            // Sidebar navigation
            Sidebar { sidebar_open: sidebar_open() }

            // Main content area
            div { class: "flex-1 flex flex-col min-w-0",
                Header {
                    sidebar_open: sidebar_open(),
                    on_sidebar_toggle: move |_| {
                        let current = sidebar_open();
                        sidebar_open.set(!current);
                    },
                }

                // Page content with fade-in transition
                main { class: "flex-1 px-4 sm:px-6 lg:px-8 py-6 overflow-auto",
                    div { class: "page-enter",
                        Outlet::<Route> {}
                    }
                }
            }
        }
    }
}
