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
/* Deliberate, dark-first design tokens (HEM-05). Dark is the default; a light
   theme is served when the viewer prefers it, and an explicit data-theme on the
   root wins in either direction. Both themes pass the WCAG-AA text matrix
   enforced in tests/console_shell.rs. */
:root {
    --surface-0: #14171c; --surface-1: #1c2027; --surface-2: #242a33;
    --ink-1: #e7eaee; --ink-2: #a9b1bc; --ink-3: #8993a1;
    --rule: #3f4856; --interactive: #7fa6e8; --focus-ring: #7fa6e8;
}
:root[data-theme="dark"] {
    --surface-0: #14171c; --surface-1: #1c2027; --surface-2: #242a33;
    --ink-1: #e7eaee; --ink-2: #a9b1bc; --ink-3: #8993a1;
    --rule: #3f4856; --interactive: #7fa6e8; --focus-ring: #7fa6e8;
}
@media (prefers-color-scheme: light) {
    :root:not([data-theme="dark"]) {
        --surface-0: #fbfcfd; --surface-1: #f2f4f7; --surface-2: #e7ebf0;
        --ink-1: #1b2027; --ink-2: #47505c; --ink-3: #5c6673;
        --rule: #cbd2dc; --interactive: #2f5fb0; --focus-ring: #2f5fb0;
    }
}
:root[data-theme="light"] {
    --surface-0: #fbfcfd; --surface-1: #f2f4f7; --surface-2: #e7ebf0;
    --ink-1: #1b2027; --ink-2: #47505c; --ink-3: #5c6673;
    --rule: #cbd2dc; --interactive: #2f5fb0; --focus-ring: #2f5fb0;
}
/* Constrain the shell to the viewport so the main content area (overflow-auto)
   becomes the scroll container, instead of the whole document growing past the
   bottom of the window (which left the page unscrollable). */
.app-shell { height: 100vh; overflow: hidden; }
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
.console-notice { display: block; margin-top: .75rem; border-style: dashed; color: var(--ink-2); }
.console-notice strong { color: var(--ink-1); }
.inspector-import { display: block; margin-top: 1rem; }
.inspector-import textarea { width: 100%; box-sizing: border-box; background: var(--surface-0); color: var(--ink-1); border: 1px solid var(--rule); padding: .75rem; font-family: 'IBM Plex Mono', ui-monospace, monospace; }
.inspector-import input { width: 100%; box-sizing: border-box; background: var(--surface-0); color: var(--ink-1); border: 1px solid var(--rule); padding: .75rem; font-family: 'IBM Plex Mono', ui-monospace, monospace; }
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
.fixture-layout { display: grid; grid-template-columns: minmax(15rem, .4fr) minmax(0, 1fr); gap: .75rem; margin-top: 1rem; }
.fixture-list { display: flex; flex-direction: column; gap: .25rem; }
.fixture-list .console-action { width: 100%; margin: 0; flex-direction: column; align-items: flex-start; }
.fixture-list .console-action small { color: var(--ink-2); }
.fixture-list [aria-pressed="true"] { color: var(--surface-0); background: var(--interactive); }
.fixture-list [aria-pressed="true"] small { color: var(--surface-0); }
.replay-demo { margin-top: 1rem; }
.assurance-inspector textarea { width: 100%; resize: vertical; border: 1px solid var(--rule); background: var(--surface-0); color: var(--ink-1); padding: .75rem; font-family: 'IBM Plex Mono', ui-monospace, monospace; }
.assurance-inspector textarea:focus-visible, .assurance-table:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
.assurance-context { margin-top: 1rem; }
.assurance-context dd { overflow-wrap: anywhere; text-align: right; }
.assurance-table { width: 100%; margin-top: .75rem; border-collapse: collapse; background: var(--surface-1); }
.assurance-table caption { padding: .75rem 0; text-align: left; font-weight: 600; }
.assurance-table th, .assurance-table td { padding: .75rem; border: 1px solid var(--rule); text-align: left; vertical-align: top; }
.assurance-table th code { display: block; margin-top: .25rem; color: var(--ink-3); font-weight: 400; }
.assurance-table p { margin-top: 0; }
.assurance-status { white-space: nowrap; }
.reason-code { display: block; margin: .25rem 0; color: var(--interactive); overflow-wrap: anywhere; }
.dispute-alerts { display: grid; grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr)); gap: .75rem; margin-top: 1rem; }
.dispute-alert { border: 2px solid var(--rule); background: var(--surface-1); padding: .75rem; }
.dispute-alert h2 { margin: 0; font-size: 1rem; } .dispute-alert strong { display: block; font-size: 1.5rem; }
.dispute-gap { border-color: #d6a85f; } .dispute-withheld { border-color: #a9b1bc; border-style: dashed; } .dispute-conflict { border-color: #ef8f9c; }
.finality-lanes { display: grid; grid-template-columns: 1fr 1fr; gap: .5rem; margin: .5rem 0; }
.finality-lane { display: flex; flex-direction: column; gap: .125rem; border: 1px solid var(--rule); background: var(--surface-1); padding: .5rem; }
.finality-lane-name { font-size: .75rem; letter-spacing: .04em; text-transform: uppercase; color: var(--ink-2); }
.finality-lane-state { font-weight: 600; font-variant: small-caps; }
.finality-lane-detail { font-size: .8125rem; color: var(--ink-2); }
.finality-lane-anchored[data-state="final"] { border-color: var(--interactive); }
.finality-lane-anchored[data-state="pending"] { border-color: #d6a85f; border-style: dashed; }
.finality-lane-anchored[data-state="unavailable"], .finality-lane-anchored[data-state="none"] { border-color: #a9b1bc; border-style: dashed; }
.finality-lane-buffered[data-state="present"] { border-color: var(--interactive); }
.finality-lane-buffered[data-state="absent"] { border-color: #a9b1bc; border-style: dashed; }
.object-relationships { list-style: none; margin: .5rem 0 0; padding: 0; display: grid; gap: .375rem; }
.object-relationships li { display: flex; flex-wrap: wrap; gap: .25rem; align-items: baseline; }
.object-rel-label { color: var(--ink-2); font-size: .8125rem; }
.object-rel-link { color: var(--interactive); text-decoration: none; font-weight: 600; }
.object-rel-link:hover, .object-rel-link:focus-visible { text-decoration: underline; }
.object-id { overflow-wrap: anywhere; }
.search-form { display: flex; gap: .5rem; margin: 1rem 0; flex-wrap: wrap; }
.search-form input { flex: 1 1 20rem; }
.search-result { margin: 1rem 0; }
.search-filters { border: 0; margin: .5rem 0; padding: 0; display: flex; flex-wrap: wrap; gap: .375rem; align-items: center; }
.search-filters legend { font-size: .8125rem; color: var(--ink-2); padding: 0 .5rem 0 0; }
.search-filters [aria-pressed="true"] { color: var(--surface-0); background: var(--interactive); }
.lineage-table { width: 100%; border-collapse: collapse; margin-top: .75rem; font-size: .875rem; }
.lineage-table caption { text-align: left; color: var(--ink-2); font-size: .8125rem; margin-bottom: .375rem; }
.lineage-table th, .lineage-table td { border: 1px solid var(--rule); padding: .5rem; text-align: left; vertical-align: top; overflow-wrap: anywhere; }
.lineage-table thead th { background: var(--surface-2); }
.lineage-links { list-style: none; margin: 0; padding: 0; display: grid; gap: .25rem; }
.lineage-node-title { color: var(--ink-2); }
.portfolio-masthead { display: flex; align-items: baseline; gap: .75rem; flex-wrap: wrap; }
.boundary-badge { font-size: .75rem; color: var(--ink-2); border: 1px solid var(--rule); border-radius: 999px; padding: .125rem .625rem; }
.portfolio-summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(7rem, 1fr)); gap: .5rem; margin: 1rem 0; }
.portfolio-stat { border: 1px solid var(--rule); background: var(--surface-1); padding: .75rem; display: flex; flex-direction: column; gap: .125rem; }
.portfolio-stat-value { font-size: 1.5rem; font-weight: 600; }
.portfolio-stat-label { font-size: .75rem; text-transform: uppercase; letter-spacing: .04em; color: var(--ink-2); }
.portfolio-loader { margin: 1rem 0; }
.portfolio-entities { display: grid; gap: .75rem; }
.portfolio-entity-head { display: flex; justify-content: space-between; gap: 1rem; flex-wrap: wrap; align-items: baseline; }
.portfolio-entity-head h2 { margin: 0; font-size: 1rem; overflow-wrap: anywhere; }
.portfolio-entity-counts { font-size: .8125rem; color: var(--ink-2); }
.portfolio-tiles { list-style: none; margin: .5rem 0 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr)); gap: .5rem; }
.portfolio-tile { border: 1px solid var(--rule); background: var(--surface-2); padding: .625rem; display: flex; flex-direction: column; gap: .375rem; }
.portfolio-tile[data-disputed="true"] { border-color: #d6a85f; }
.portfolio-tile-id { text-decoration: none; color: var(--interactive); overflow-wrap: anywhere; }
.portfolio-tile-id:hover, .portfolio-tile-id:focus-visible { text-decoration: underline; }
.portfolio-tile-badges { display: flex; flex-wrap: wrap; gap: .25rem; }
.portfolio-badge { font-size: .6875rem; text-transform: uppercase; letter-spacing: .03em; border: 1px solid var(--rule); padding: .0625rem .375rem; color: var(--ink-2); }
.portfolio-badge-disputed { color: var(--ink-1); border-color: #d6a85f; }
.dispute-controls { display: flex; align-items: end; justify-content: space-between; gap: 1rem; margin: 1rem 0; flex-wrap: wrap; }
.dispute-controls fieldset { border: 0; margin: 0; padding: 0; } .dispute-controls .console-action { margin-right: .375rem; }
.dispute-controls [aria-pressed="true"] { color: var(--surface-0); background: var(--interactive); }
.evidence-graph { display: grid; grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr)); gap: .75rem; }
.evidence-node { border: 1px solid var(--interactive); background: var(--surface-1); padding: .75rem; overflow-wrap: anywhere; }
.evidence-node-gap { border: 2px solid #d6a85f; } .evidence-node-withheld { border-style: dashed; }
.evidence-node:focus-visible, .dispute-controls button:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
.edge-list { margin-top: .75rem; } .edge-list code, .dispute-table code { overflow-wrap: anywhere; }
.dispute-table { width: 100%; border-collapse: collapse; } .dispute-table caption { text-align: left; font-weight: 600; padding: .5rem 0; }
.dispute-table th, .dispute-table td { padding: .625rem; text-align: left; vertical-align: top; border: 1px solid var(--rule); }
.dispute-conflict-list { border-left: 4px solid #ef8f9c; padding: .75rem 1.75rem; background: var(--surface-1); }
/* Accountability explorer */
.console-home.explorer { max-width: none; padding-bottom: 2rem; }
.explorer-link { color: var(--interactive); cursor: pointer; text-decoration: underline; }
.explorer-link:hover { color: var(--ink-1); }
.explorer-links { display: flex; flex-wrap: wrap; gap: .75rem; margin: .35rem 0; }
.explorer-panes { display: grid; grid-template-columns: minmax(0, 22rem) minmax(0, 1fr); gap: .75rem; margin-top: 1rem; align-items: start; width: 100%; }
.explorer-pane { min-width: 0; overflow-wrap: anywhere; }
.explorer-feed-head { display: flex; align-items: center; justify-content: space-between; gap: .5rem; min-width: 0; }
.explorer-feed { list-style: none; margin: .75rem 0 0; padding: 0; display: flex; flex-direction: column; gap: .5rem; }
.explorer-feed li { min-width: 0; }
.explorer-feed-row { display: flex; flex-direction: column; align-items: stretch; gap: .25rem; width: 100%; min-width: 0; text-align: left; padding: .625rem .75rem; border: 1px solid var(--rule); border-radius: .375rem; background: var(--surface-1); color: var(--ink-1); cursor: pointer; overflow-wrap: anywhere; word-break: break-word; }
.explorer-feed-row:hover { background: var(--surface-2); }
.explorer-feed-row:focus-visible, .explorer-chip:focus-visible { outline: 2px solid var(--focus-ring); outline-offset: 2px; }
.explorer-feed-row[aria-pressed="true"] { border-color: var(--interactive); }
.explorer-feed-row strong { font-weight: 600; }
.explorer-feed-row small { color: var(--ink-2); }
.explorer-chips { display: flex; flex-wrap: wrap; gap: .35rem; margin: .35rem 0; min-width: 0; }
.explorer-chip { display: inline-block; max-width: 100%; padding: .25rem .5rem; border: 1px solid var(--interactive); border-radius: .375rem; background: transparent; color: var(--interactive); font-family: 'IBM Plex Mono', ui-monospace, monospace; font-size: .75rem; line-height: 1.4; text-align: left; cursor: pointer; word-break: break-all; overflow-wrap: anywhere; }
.explorer-chip:hover { background: var(--surface-2); }
.explorer-chip-static { color: var(--ink-2); border-color: var(--rule); cursor: default; }
.explorer-badge { display: inline-block; flex: none; padding: .05rem .4rem; border: 1px solid var(--rule); border-radius: 999px; font-size: .7rem; color: var(--ink-2); }
.explorer-card { border: 1px solid var(--rule); border-radius: .5rem; background: var(--surface-1); padding: .75rem; margin: .5rem 0; min-width: 0; overflow-wrap: anywhere; word-break: break-word; }
.explorer-card h3 { margin: 0 0 .25rem; font-size: .95rem; font-weight: 600; }
.explorer-card p { margin: .2rem 0; }
.explorer-card code, .explorer-feed-row code, .explorer-timeline code { word-break: break-all; overflow-wrap: anywhere; }
.explorer-section-title { margin: 1rem 0 .25rem; font-size: .9rem; font-weight: 600; color: var(--ink-1); }
.explorer-timeline { margin: .25rem 0 .5rem; padding-left: 1.1rem; }
.explorer-timeline li { padding: .25rem 0; overflow-wrap: anywhere; word-break: break-word; }
.explorer-timeline small { color: var(--ink-2); }
.explorer-pager { display: flex; align-items: center; justify-content: center; gap: .75rem; margin-top: .5rem; flex-wrap: wrap; }
.explorer-pager .console-action { margin-top: 0; }
.explorer-pager button:disabled { color: var(--ink-3); border-color: var(--rule); cursor: not-allowed; }
.explorer-chain { margin-top: .5rem; }
@media (max-width: 900px) { .explorer-panes { grid-template-columns: 1fr; } }
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
    .fixture-layout { grid-template-columns: 1fr; }
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
