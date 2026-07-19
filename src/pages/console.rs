//! Hemion developer-console home (Flow Spec S-H1).

use crate::routes::Route;
use dioxus::prelude::*;

/// The local console entry point. It advertises only capabilities implemented
/// in this ticket and keeps externally recorded verdicts explicitly non-local.
#[component]
pub fn ConsoleHome() -> Element {
    rsx! {
        section { class: "console-home", aria_labelledby: "console-title",
            p { class: "console-eyebrow", "HEMION / LOCAL INSTRUMENT" }
            h1 { id: "console-title", "Developer console" }
            p { class: "console-lede",
                "Inspect Parwana artifacts locally. Hemion accepts no verdict merely because another service recorded it."
            }

            div { class: "console-grid",
                article { class: "console-panel",
                    h2 { "Local verification" }
                    dl {
                        div { dt { "Verifier" } dd { "Not loaded" } }
                        div { dt { "Contract" } dd { class: "console-mono", "csv-sdk =0.1.5" } }
                        div { dt { "Trust packages loaded" } dd { "None" } }
                        div { dt { "Locally verified objects" } dd { "None" } }
                    }
                    p { class: "console-limitation",
                        span { aria_hidden: "true", "◇ " }
                        "No local accountability bundle has been verified in this session."
                    }
                }

                article { class: "console-panel",
                    h2 { "Available tools" }
                    p { "The existing wallet is preserved as a separate legacy tool area." }
                    Link { to: Route::Dashboard {}, class: "console-action", "Open legacy wallet" }
                }
            }

            aside { class: "console-notice", aria_label: "Authority limitation",
                strong { "Recorded elsewhere is not locally verified." }
                span { " Bundle import and assurance inspection will appear only when Hemion can compute their results locally." }
            }
        }
    }
}
