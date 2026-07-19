#![allow(clippy::needless_pass_by_value)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
//! Hemion — local Parwana developer console with preserved legacy wallet tools.

#![warn(missing_docs)]
#![allow(non_snake_case)]

use dioxus::prelude::*;

mod chains;
mod components;
mod context;
mod core;
mod hooks;
mod layout;
mod pages;
mod routes;
mod services;
mod storage;
mod ui_error;
mod wallet;
mod wallet_core;

use context::WalletProvider;
use routes::Route;

const TAILWIND_CSS: &str = include_str!("../assets/tailwind.css");

fn main() {
    console_error_panic_hook::set_once();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // Tailwind CSS stylesheet (embedded)
        document::Style {
            r#type: "text/css",
            "{TAILWIND_CSS}"
        }

        // Critical reset (required for base styling)
        document::Style {
            r#type: "text/css",
            "{CRITICAL_CSS}"
        }

        // Animations and transitions
        document::Style {
            r#type: "text/css",
            "{GLOBAL_CSS}"
        }

        // Main app with all required providers
        WalletProvider {
            hooks::WalletProvider {
                hooks::NetworkProvider {
                    hooks::BalanceProvider {
                        hooks::WalletConnectionProvider {
                            Router::<Route> {}
                        }
                    }
                }
            }
        }
    }
}

const CRITICAL_CSS: &str = "
*, *::before, *::after { box-sizing: border-box; }
body { min-height: 100vh; margin: 0; padding: 0; background: #14171c; color: #e7eaee; font-family: 'IBM Plex Sans', system-ui, sans-serif; }
#main { display: block; }
";

const GLOBAL_CSS: &str = r#"
:root {
    --surface-0: #14171c; --surface-1: #1c2027; --surface-2: #242a33;
    --ink-1: #e7eaee; --ink-2: #a9b1bc; --ink-3: #8993a1;
    --rule: #3f4856; --interactive: #7fa6e8; --focus-ring: #7fa6e8;
}
.instrument-sidebar { width: 16rem; min-height: 100vh; position: sticky; top: 0; flex-shrink: 0; display: flex; flex-direction: column; background: var(--surface-1); border-right: 1px solid var(--rule); }
.instrument-sidebar-closed { width: 4rem; }
.instrument-brand, .instrument-nav-link, .instrument-nav-tab, .console-action { color: var(--ink-1); text-decoration: none; }
.instrument-brand { min-height: 4rem; padding: .75rem; display: flex; align-items: center; gap: .5rem; border-bottom: 1px solid var(--rule); font-size: 1.125rem; font-weight: 600; }
.instrument-nav { display: flex; flex: 1; flex-direction: column; gap: .25rem; padding: .5rem; }
.instrument-nav-heading, .console-eyebrow { margin: .75rem .5rem .25rem; color: var(--ink-3); font: 500 .75rem/1.4 ui-monospace, monospace; letter-spacing: .08em; text-transform: uppercase; }
.instrument-nav-link { min-height: 2.75rem; padding: .5rem .75rem; display: flex; align-items: center; gap: .75rem; border-radius: .375rem; }
.instrument-nav-link:hover, .instrument-nav-tab:hover, .console-action:hover { background: var(--surface-2); }
.instrument-boundary { padding: .75rem; color: var(--ink-3); font-size: .75rem; line-height: 1.5; }
.instrument-tabs { display: none; }
.instrument-brand:focus-visible, .instrument-nav-link:focus-visible, .instrument-nav-tab:focus-visible, .console-action:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
.console-home { max-width: 72rem; margin: 0 auto; color: var(--ink-1); }
.console-home h1 { margin: .25rem 0; font-size: 1.953rem; font-weight: 600; }
.console-lede { max-width: 48rem; color: var(--ink-2); }
.console-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr)); gap: .75rem; margin-top: 1.5rem; }
.console-panel, .console-notice { border: 1px solid var(--rule); background: var(--surface-1); padding: 1rem; }
.console-panel h2 { margin-top: 0; font-size: 1rem; font-weight: 600; }
.console-panel dl div { display: flex; justify-content: space-between; gap: 1rem; padding: .5rem 0; border-top: 1px solid var(--rule); }
.console-panel dt { color: var(--ink-2); } .console-panel dd { margin: 0; }
.console-mono { font-family: 'IBM Plex Mono', ui-monospace, monospace; }
.console-limitation { color: var(--ink-3); font-size: .8125rem; }
.console-action { display: inline-flex; min-height: 2.75rem; align-items: center; margin-top: .5rem; padding: .5rem .75rem; border: 1px solid var(--interactive); border-radius: .375rem; color: var(--interactive); }
.console-notice { margin-top: .75rem; border-style: dashed; color: var(--ink-2); }
.console-notice strong { color: var(--ink-1); }
.inspector-import { display: block; margin-top: 1rem; }
.inspector-import textarea { width: 100%; box-sizing: border-box; background: var(--surface-0); color: var(--ink-1); border: 1px solid var(--rule); padding: .75rem; font-family: 'IBM Plex Mono', ui-monospace, monospace; }
.inspector-columns { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: .75rem; margin-top: 1rem; }
.inspector-field { margin: 0; }
.inspector-field div { display: grid !important; grid-template-columns: minmax(9rem, .35fr) minmax(0, 1fr); }
.inspector-field dd, .inspector-bytes pre { overflow-wrap: anywhere; white-space: pre-wrap; }
.inspector-bytes summary { min-height: 2.75rem; display: flex; align-items: center; color: var(--interactive); cursor: pointer; }
.inspector-bytes summary:focus-visible, .inspector-evidence:focus-visible, .inspector-import textarea:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
.inspector-timeline { margin-top: .75rem; }
.inspector-timeline li { display: grid; grid-template-columns: 11rem 12rem minmax(10rem, 1fr); gap: .75rem; padding: .5rem; border-top: 1px solid var(--rule); }
.inspector-timeline li span { overflow-wrap: anywhere; }
.inspector-evidence { padding: .75rem; border-top: 1px solid var(--rule); }
.inspector-evidence h3 { margin: 0; }
.bundle-verify textarea { width: 100%; resize: vertical; border: 1px solid var(--rule); background: var(--surface-0); color: var(--ink-1); padding: .75rem; font-family: 'IBM Plex Mono', ui-monospace, monospace; }
.bundle-verify textarea:focus-visible, .bundle-verify button:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
.bundle-verify button:disabled { color: var(--ink-3); border-color: var(--rule); cursor: not-allowed; }
/* Page Transitions */
.page-enter {
    animation: pageFadeIn 0.3s ease-out;
}
@keyframes pageFadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
}

/* Stagger Children */
.stagger-children > * {
    animation: staggerFadeIn 0.4s ease-out backwards;
}
.stagger-children > *:nth-child(1) { animation-delay: 0.05s; }
.stagger-children > *:nth-child(2) { animation-delay: 0.1s; }
.stagger-children > *:nth-child(3) { animation-delay: 0.15s; }
.stagger-children > *:nth-child(4) { animation-delay: 0.2s; }
.stagger-children > *:nth-child(5) { animation-delay: 0.25s; }
.stagger-children > *:nth-child(6) { animation-delay: 0.3s; }
.stagger-children > *:nth-child(7) { animation-delay: 0.35s; }
.stagger-children > *:nth-child(8) { animation-delay: 0.4s; }
@keyframes staggerFadeIn {
    from { opacity: 0; transform: translateY(12px); }
    to { opacity: 1; transform: translateY(0); }
}

/* Preserve motion preferences and keep the desktop-first shell usable on narrow screens. */
@media (prefers-reduced-motion: reduce) {
    *, *::before, *::after { animation-duration: 0.01ms !important; animation-iteration-count: 1 !important; scroll-behavior: auto !important; transition-duration: 0.01ms !important; }
}
@media (max-width: 767px) {
    .inspector-columns { grid-template-columns: 1fr; }
    .inspector-timeline li { grid-template-columns: 1fr; }
    .app-sidebar { display: none; }
    .instrument-sidebar { display: none; }
    .instrument-tabs { position: fixed; inset: auto 0 0; z-index: 60; display: flex; background: var(--surface-1); border-top: 1px solid var(--rule); }
    .instrument-nav-tab { min-height: 2.75rem; flex: 1; padding: .5rem; display: flex; align-items: center; justify-content: center; gap: .5rem; font-size: .75rem; }
    .app-header-content { min-height: 0; padding-top: 0.5rem; padding-bottom: 0.5rem; align-items: flex-start; }
    .app-header-controls { flex: 1; justify-content: flex-end; flex-wrap: wrap; gap: 0.5rem; }
    .app-header-controls > div { gap: 0.25rem; }
    .wallet-page-title, .account-row { align-items: flex-start; flex-direction: column; }
    .account-actions { flex-wrap: wrap; }
}

/* Button Ripple */
.btn-ripple {
    position: relative;
    overflow: hidden;
}
.btn-ripple::after {
    content: '';
    position: absolute;
    top: 50%; left: 50%;
    width: 0; height: 0;
    border-radius: 50%;
    background: rgba(255,255,255,0.15);
    transform: translate(-50%,-50%);
    transition: width 0.4s ease, height 0.4s ease, opacity 0.4s ease;
    opacity: 0;
}
.btn-ripple:active::after {
    width: 200px; height: 200px; opacity: 1; transition: 0s;
}

/* Card Hover */
.card-hover {
    transition: transform 0.2s ease, box-shadow 0.2s ease;
}
.card-hover:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}

/* Pulse Glow */
.pulse-glow {
    animation: pulseGlow 2s ease-in-out infinite;
}
@keyframes pulseGlow {
    0%,100% { box-shadow: 0 0 4px rgba(59,130,246,0.3); }
    50% { box-shadow: 0 0 16px rgba(59,130,246,0.6); }
}

/* Status Pulse */
.status-online {
    animation: statusPulse 2s ease-in-out infinite;
}
@keyframes statusPulse {
    0%,100% { opacity: 1; }
    50% { opacity: 0.5; }
}

/* Count Up */
.count-up {
    animation: countUp 0.5s ease-out;
}
@keyframes countUp {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
}

/* Input Focus */
.input-focus {
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
}
.input-focus:focus {
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59,130,246,0.15);
    outline: none;
}

/* Modal */
.modal-backdrop {
    animation: backdropFadeIn 0.2s ease-out;
}
@keyframes backdropFadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}
.modal-content {
    animation: modalSlideIn 0.25s ease-out;
}
@keyframes modalSlideIn {
    from { opacity: 0; transform: scale(0.95) translateY(-10px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
}

/* Scrollbar */
::-webkit-scrollbar { width: 8px; height: 8px; }
::-webkit-scrollbar-track { background: #111827; border-radius: 4px; }
::-webkit-scrollbar-thumb { background: #374151; border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: #4b5563; }

/* Selection */
::selection {
    background: rgba(59,130,246,0.3);
    color: #f3f4f6;
}

/* Smooth scroll */
html { scroll-behavior: smooth; }
"#;
