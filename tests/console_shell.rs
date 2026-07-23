//! G-01 navigation, product identity, and accessibility regression tests.

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).expect("fixture must exist")
}

fn luminance(hex: &str) -> f64 {
    let channel = |offset| {
        let value = u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap() as f64 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5)
}

fn contrast(a: &str, b: &str) -> f64 {
    let (light, dark) = if luminance(a) > luminance(b) {
        (luminance(a), luminance(b))
    } else {
        (luminance(b), luminance(a))
    };
    (light + 0.05) / (dark + 0.05)
}

#[test]
fn wcag_aa_console_text_matrix_passes() {
    let pairs = [
        ("ink-1/surface-0", "#e7eaee", "#14171c"),
        ("ink-2/surface-0", "#a9b1bc", "#14171c"),
        ("ink-3/surface-0", "#8993a1", "#14171c"),
        ("ink-1/surface-1", "#e7eaee", "#1c2027"),
        ("ink-2/surface-1", "#a9b1bc", "#1c2027"),
        ("ink-3/surface-1", "#8993a1", "#1c2027"),
        ("interactive/surface-0", "#7fa6e8", "#14171c"),
        ("interactive/surface-1", "#7fa6e8", "#1c2027"),
    ];
    for (name, foreground, background) in pairs {
        assert!(
            contrast(foreground, background) >= 4.5,
            "{name} is only {:.2}:1",
            contrast(foreground, background)
        );
    }
    for (name, foreground, background) in [
        ("gap-border/surface-1", "#d6a85f", "#1c2027"),
        ("conflict-border/surface-1", "#ef8f9c", "#1c2027"),
    ] {
        assert!(
            contrast(foreground, background) >= 3.0,
            "{name} is only {:.2}:1",
            contrast(foreground, background)
        );
    }
}

#[test]
fn portfolio_is_the_home_and_console_and_wallet_remain_routable() {
    let routes = read("src/routes.rs");
    // HEM-05: the portfolio-of-mandates home is the default route; the developer
    // console moves to /console but stays reachable, and the wallet keeps its
    // routes.
    assert!(routes.contains("#[route(\"/\")]\n    PortfolioHome"));
    assert!(routes.contains("#[route(\"/console\")]\n    ConsoleHome"));
    assert!(routes.contains("#[route(\"/wallet\")]\n    Dashboard"));
    for route in ["/assets", "/activity", "/contacts", "/settings"] {
        assert!(routes.contains(route));
    }
}

#[test]
fn wcag_aa_console_text_matrix_passes_light_theme() {
    // HEM-05 adds a light theme; its text tokens must also clear WCAG AA (4.5:1)
    // against the light surfaces.
    let pairs = [
        ("ink-1/surface-0", "#1b2027", "#fbfcfd"),
        ("ink-2/surface-0", "#47505c", "#fbfcfd"),
        ("ink-3/surface-0", "#5c6673", "#fbfcfd"),
        ("ink-1/surface-1", "#1b2027", "#f2f4f7"),
        ("ink-2/surface-1", "#47505c", "#f2f4f7"),
        ("ink-3/surface-1", "#5c6673", "#f2f4f7"),
        ("interactive/surface-0", "#2f5fb0", "#fbfcfd"),
        ("interactive/surface-1", "#2f5fb0", "#f2f4f7"),
    ];
    for (name, foreground, background) in pairs {
        assert!(
            contrast(foreground, background) >= 4.5,
            "light {name} is only {:.2}:1",
            contrast(foreground, background)
        );
    }
    // The light token block must actually be present in the stylesheet.
    let css = read("src/main.rs");
    assert!(css.contains("prefers-color-scheme: light"));
    assert!(css.contains(":root[data-theme=\"light\"]"));
    assert!(css.contains("background: var(--surface-0, #14171c)"));
    assert!(css.contains(".app-shell { height: 100vh; overflow: hidden; background: var(--surface-0); color: var(--ink-1); }"));
}

#[test]
fn implemented_console_inspectors_are_exposed() {
    let navigation = read("src/components/sidebar.rs");
    assert!(navigation.contains("label: \"Bundle verifier\""));
    assert!(navigation.contains("label: \"Object inspector\""));
    assert!(navigation.contains("label: \"Dispute inspector\""));
    assert!(navigation.contains("label: \"Assurance inspector\""));
    assert!(navigation.contains("aria_label: destination.label"));
    let css = read("src/main.rs");
    assert!(css.contains(":focus-visible { outline: 2px"));
    assert!(css.contains("prefers-reduced-motion: reduce"));
}

#[test]
fn assurance_inspector_shows_all_dimensions_context_reasons_and_limitations() {
    let page = read("src/pages/assurance_inspector.rs");
    for required in [
        "All 11 checks under the selected context",
        "Context digest",
        "Verifier policy digest",
        "Trust package digest",
        "Reason codes and meaning",
        "Limitations",
        "Cannot be determined",
        "Not applicable",
        "External source support",
        "tabindex: \"0\"",
    ] {
        assert!(
            page.contains(required),
            "missing assurance requirement: {required}"
        );
    }
    for prohibited in ["independent confirmation", "verified ✓", "risk score"] {
        assert!(
            !page.contains(prohibited),
            "misleading assurance language: {prohibited}"
        );
    }
    assert!(page.contains("no single trust score is produced"));
    assert!(page.contains("import_and_verify"));
}

#[test]
fn dispute_inspector_has_filters_prominent_uncertainty_and_table_alternative() {
    let page = read("src/pages/dispute_inspector.rs");
    for required in [
        "Filter node types",
        "Evidence gaps",
        "Withheld branches",
        "Potential contradictions",
        "Show accessible table view",
        "aria_pressed",
        "scope: \"col\"",
    ] {
        assert!(
            page.contains(required),
            "missing dispute UI requirement: {required}"
        );
    }
    assert!(page.contains("Triage only"));
    assert!(page.contains("never establishes non-occurrence"));
    for prohibited in ["Authorized", "confirmed true", "No evidence means"] {
        assert!(
            !page.contains(prohibited),
            "misleading dispute language: {prohibited}"
        );
    }
}

#[test]
fn object_inspector_is_read_only_accessible_and_evidence_first() {
    let page = read("src/pages/object_inspector.rs");
    assert!(page.contains("Canonical bytes"));
    assert!(page.contains("Replay timeline"));
    assert!(page.contains("ActionMandate"));
    assert!(page.contains("ExecutionReceipt"));
    assert!(page.contains("Absence of a row does not establish non-occurrence."));
    assert!(page.contains("tabindex: \"0\""));
    for prohibited in ["Approve & sign", "Execute mandate", "Mark consumed"] {
        assert!(
            !page.contains(prohibited),
            "inspector exposes mutation: {prohibited}"
        );
    }
}

#[test]
fn product_metadata_and_local_authority_boundary_are_explicit() {
    assert!(read("Cargo.toml").contains("Hemion developer console"));
    assert!(read("Dioxus.toml").contains("category = \"DeveloperTool\""));
    let home = read("src/pages/console.rs");
    assert!(home.contains("Recorded elsewhere is not locally verified."));
    assert!(!home.contains("verified ✓"));
}

#[test]
fn tuppira_explorer_keeps_discovery_and_local_verification_distinct() {
    let page = read("src/pages/tuppira_explorer.rs");
    for required in [
        "Observation discovery and lineage",
        "Source health",
        "Discover and trace",
        "Recorded elsewhere · not locally verified",
        "Verify selected evidence locally",
        "Absence does not establish non-occurrence",
    ] {
        assert!(page.contains(required), "missing G-08 behavior: {required}");
    }
    assert!(page.contains("verify_selected"));
    assert!(!page.contains("Tuppira verified"));
}

#[test]
fn fixture_lab_is_browsable_accessible_and_computes_actual_results_locally() {
    let page = read("src/pages/fixture_lab.rs");
    for required in [
        "Expected",
        "Actual",
        "aria_pressed",
        "aria_live",
        "Run selected vector locally",
        "First attempt — accepted once.",
        "Second attempt — rejected.",
        "ReplayDetected",
        "no provider dispatch was performed",
        "Missing evidence never establishes non-occurrence",
    ] {
        assert!(
            page.contains(required),
            "missing fixture-lab requirement: {required}"
        );
    }
    assert!(page.contains("run_case"));
    assert!(page.contains("import_context"));
    for prohibited in ["simulated success", "mark consumed", "dispatch_provider"] {
        assert!(!page.contains(prohibited));
    }
    let navigation = read("src/components/sidebar.rs");
    assert!(navigation.contains("label: \"Fixture lab\""));
}
