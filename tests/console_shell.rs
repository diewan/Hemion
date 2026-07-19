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
}

#[test]
fn console_is_primary_and_legacy_wallet_remains_routable() {
    let routes = read("src/routes.rs");
    assert!(routes.contains("#[route(\"/\")]\n    ConsoleHome"));
    assert!(routes.contains("#[route(\"/wallet\")]\n    Dashboard"));
    for route in ["/assets", "/activity", "/contacts", "/settings"] {
        assert!(routes.contains(route));
    }
}

#[test]
fn bundle_verifier_is_exposed_but_later_inspectors_are_not() {
    let navigation = read("src/components/sidebar.rs");
    assert!(navigation.contains("label: \"Bundle verifier\""));
    for absent in ["Assurance inspector", "Object inspector"] {
        assert!(
            !navigation.contains(&format!("label: \"{absent}")),
            "unfinished destination exposed: {absent}"
        );
    }
    assert!(navigation.contains("aria_label: destination.label"));
    let css = read("src/main.rs");
    assert!(css.contains(":focus-visible { outline: 2px"));
    assert!(css.contains("prefers-reduced-motion: reduce"));
}

#[test]
fn product_metadata_and_local_authority_boundary_are_explicit() {
    assert!(read("Cargo.toml").contains("Hemion developer console"));
    assert!(read("Dioxus.toml").contains("category = \"DeveloperTool\""));
    let home = read("src/pages/console.rs");
    assert!(home.contains("Recorded elsewhere is not locally verified."));
    assert!(!home.contains("verified ✓"));
}
