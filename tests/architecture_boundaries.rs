//! Regression coverage for wallet dependency and import boundaries.
//!
//! The wallet may use the SDK facade, but must never recover direct authority
//! over retired protocol code or concrete chain adapters.

use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_MANIFEST_DEPENDENCIES: &[&str] = &[
    "csv-core",
    "csv-runtime",
    "csv-store",
    "csv-storage",
    "csv-wallet",
    "csv-keys",
    "csv-wire",
    "csv-codec",
    "csv-protocol",
    "csv-hash",
    "csv-proof",
    "csv-verifier",
    "csv-coordinator",
    "csv-admission",
    "csv-adapter-factory",
    "tuppira-shared",
    "csv-bitcoin",
    "csv-ethereum",
    "csv-solana",
    "csv-sui",
    "csv-aptos",
    "csv-celestia",
    "csv-adapters/",
];

const FORBIDDEN_SOURCE_IMPORTS: &[&str] = &[
    "csv_core::",
    "csv_runtime::",
    "csv_store::",
    "csv_storage::",
    "csv_wallet::",
    "csv_keys::",
    "csv_wire::",
    "csv_codec::",
    "csv_protocol::",
    "csv_hash::",
    "csv_proof::",
    "csv_verifier::",
    "tuppira_shared::",
    "use csv_bitcoin::",
    "use csv_ethereum::",
    "use csv_solana::",
    "use csv_sui::",
    "use csv_aptos::",
    "use csv_celestia::",
    "csv_sdk::csv_bitcoin::",
    "csv_sdk::csv_ethereum::",
    "csv_sdk::csv_solana::",
    "csv_sdk::csv_sui::",
    "csv_sdk::csv_aptos::",
    "csv_sdk::csv_celestia::",
];

const REQUIRED_MANIFEST_DEPENDENCIES: &[&str] = &["csv-sdk"];

const REQUIRED_SOURCE_MARKERS: &[&str] = &[
    "csv_sdk::canonical::",
    "csv_sdk::verification::",
    "csv_sdk::protocol::",
];

const FORBIDDEN_LOCAL_AUTHORITY_MARKERS: &[&str] = &[
    ".consume_seal(",
    ".lock_seal(",
    "seal.status = SealStatus::Consumed",
    "seal.status = SealStatus::Locked",
    "pub struct SealManager",
    "build_transaction(",
];

const FORBIDDEN_PLATFORM_PLACEHOLDERS: &[&str] = &[
    "WASM32 stub",
    "placeholder balance",
    "returns empty ChainApi",
];

const FORBIDDEN_PRODUCTION_SIMULATION_MARKERS: &[&str] = &[
    "proof_data_value",
    "Proof is valid.",
    "Commitment chain is valid.",
    "All tests completed",
    "Scenario '{}' completed successfully.",
    "Route::Test",
    "add_test_result",
];

#[test]
fn wallet_has_no_legacy_or_direct_adapter_dependencies() {
    let manifest = fs::read_to_string(manifest_path()).expect("wallet manifest must be readable");
    let dependency_lines = active_dependency_lines(&manifest);

    let violations: Vec<_> = FORBIDDEN_MANIFEST_DEPENDENCIES
        .iter()
        .filter(|dependency| {
            dependency_lines.iter().any(|line| {
                line.starts_with(&format!("{dependency} "))
                    || line.starts_with(&format!("{dependency}="))
                    || (**dependency == "csv-adapters/" && line.contains(*dependency))
            })
        })
        .map(|dependency| format!("Cargo.toml contains forbidden dependency marker `{dependency}`"))
        .collect();

    assert!(
        violations.is_empty(),
        "wallet dependencies must use the runtime/SDK boundary:\n{}",
        violations.join("\n")
    );
}

#[test]
fn wallet_source_has_no_legacy_or_direct_adapter_imports() {
    let violations = scan_rust_files(&source_root(), FORBIDDEN_SOURCE_IMPORTS);

    assert!(
        violations.is_empty(),
        "wallet source must not import retired csv-core or concrete adapters:\n{}",
        violations.join("\n")
    );
}

#[test]
fn wallet_declares_and_uses_the_canonical_transport_boundary() {
    let manifest = fs::read_to_string(manifest_path()).expect("wallet manifest must be readable");
    let dependency_lines = active_dependency_lines(&manifest);
    for dependency in REQUIRED_MANIFEST_DEPENDENCIES {
        assert!(
            dependency_lines
                .iter()
                .any(|line| line.starts_with(&format!("{dependency} "))
                    || line.starts_with(&format!("{dependency}="))),
            "wallet must directly declare `{dependency}`"
        );
    }

    let violations = missing_source_markers(&source_root(), REQUIRED_SOURCE_MARKERS);
    assert!(
        violations.is_empty(),
        "wallet must use canonical wire and codec paths:\n{}",
        violations.join("\n")
    );
}

#[test]
fn sdk_dependency_and_contract_are_exactly_pinned() {
    let manifest = fs::read_to_string(manifest_path()).expect("wallet manifest must be readable");
    assert!(manifest.contains("csv-sdk = { version = \"=0.1.5\""));
    assert!(!manifest.contains("[patch."));
    assert!(!manifest.contains("git ="));

    let pin = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".diewan/parwana-contract.toml"),
    )
    .expect("Parwana contract pin must be readable");
    assert!(pin.contains("contract_version = \"0.1.5\""));
    let commit = pin
        .lines()
        .find(|line| line.starts_with("value = \""))
        .expect("contract source commit must be present");
    assert_eq!(commit.len(), "value = \"\"".len() + 40);
}

#[test]
fn feature_sets_exclude_concrete_and_authority_capabilities() {
    let manifest = fs::read_to_string(manifest_path()).expect("wallet manifest must be readable");
    for forbidden in [
        "runtime-coordinator",
        "all-chains",
        "csv-sdk/bitcoin",
        "csv-sdk/ethereum",
        "csv-sdk/solana",
        "csv-sdk/sui",
        "csv-sdk/aptos",
        "csv-sdk/p2p",
        "csv-sdk/sqlite",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "forbidden SDK capability `{forbidden}`"
        );
    }
}

#[test]
fn wallet_has_no_local_transfer_or_seal_authority() {
    let violations = scan_rust_files(&source_root(), FORBIDDEN_LOCAL_AUTHORITY_MARKERS);
    assert!(
        violations.is_empty(),
        "wallet must not mutate lifecycle state locally:\n{}",
        violations.join("\n")
    );
}

#[test]
fn wallet_has_no_shippable_simulation_or_prototype_product() {
    for prototype in ["hemion-core", "hemion-ui"] {
        assert!(
            !Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(prototype)
                .exists(),
            "retired wallet prototype `{prototype}` must not remain in the repository"
        );
    }

    let violations = scan_rust_files(&source_root(), FORBIDDEN_PRODUCTION_SIMULATION_MARKERS);
    assert!(
        violations.is_empty(),
        "wallet must not ship simulated proof, test, or success workflows:\n{}",
        violations.join("\n")
    );
}

#[test]
fn wallet_import_export_uses_only_the_shared_encrypted_envelope() {
    let source = fs::read_to_string(source_root().join("context/wallet.rs"))
        .expect("wallet context must be readable");
    let page = fs::read_to_string(source_root().join("pages/wallet_page.rs"))
        .expect("wallet page must be readable");
    let platform = fs::read_to_string(source_root().join("services/platform.rs"))
        .expect("platform file adapter must be readable");

    assert!(
        source.contains("format::encrypt") && source.contains("format::decrypt"),
        "wallet import/export must use the shared wallet-format envelope"
    );
    assert!(
        !source.contains("import_wallet_json") && !source.contains("export_wallet_json"),
        "wallet must not retain a plaintext JSON import/export path"
    );
    assert!(
        page.contains("read_bytes().await")
            && !platform.contains("read_as_text")
            && !page.contains("read_as_text"),
        "the web file adapter must keep the encrypted file as bytes rather than UI text"
    );
}

#[test]
fn presentation_uses_explicit_platform_ports_without_wasm_placeholders() {
    let platform = fs::read_to_string(source_root().join("services/platform.rs"))
        .expect("platform ports must be readable");
    let chain_api = fs::read_to_string(source_root().join("services/chain_api.rs"))
        .expect("chain API compatibility export must be readable");
    let transfer_page = fs::read_to_string(source_root().join("pages/cross_chain/transfer.rs"))
        .expect("transfer page must be readable");

    for marker in [
        "trait RuntimePort",
        "trait VaultPort",
        "trait CanonicalIntentPort",
    ] {
        assert!(
            platform.contains(marker),
            "platform contract lacks `{marker}`"
        );
    }
    assert!(
        platform.contains("validate_signing_intent")
            && platform.contains("binding_digest")
            && platform.contains("NetworkMismatch"),
        "typed local signing must validate complete, bound, network-matching intents"
    );
    assert!(
        platform.contains("RemoteRuntimeUnavailable")
            && platform.contains("UnsupportedCapability")
            && platform.contains("RuntimeRequest")
            && platform.contains("/v1/wallet/runtime"),
        "web capability gaps must be typed errors"
    );
    assert!(
        !chain_api.contains("csv_sdk::runtime") && transfer_page.contains("WalletPlatform"),
        "Dioxus presentation must use the neutral platform facade, not a native runtime"
    );

    let violations: Vec<_> = FORBIDDEN_PLATFORM_PLACEHOLDERS
        .iter()
        .filter(|marker| chain_api.contains(**marker))
        .map(|marker| format!("chain API compatibility export contains `{marker}`"))
        .collect();
    assert!(
        violations.is_empty(),
        "wallet must not retain wasm placeholder behavior:\n{}",
        violations.join("\n")
    );
}

#[test]
fn page_code_has_no_target_arch_branches() {
    let violations = scan_rust_files(
        &source_root().join("pages"),
        &["cfg(target_arch", "cfg_attr(target_arch"],
    );
    assert!(
        violations.is_empty(),
        "page code must consume platform ports instead of selecting targets:\n{}",
        violations.join("\n")
    );
}

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

fn source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn active_dependency_lines(manifest: &str) -> Vec<&str> {
    let mut in_dependencies = false;

    manifest
        .lines()
        .filter_map(|line| {
            let active = line
                .split_once('#')
                .map_or(line, |(active, _)| active)
                .trim();
            if active.starts_with('[') && active.ends_with(']') {
                in_dependencies = active.ends_with("dependencies]");
                return None;
            }
            (in_dependencies && !active.is_empty()).then_some(active)
        })
        .collect()
}

fn scan_rust_files(root: &Path, forbidden: &[&str]) -> Vec<String> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);

    files
        .into_iter()
        .flat_map(|path| {
            let contents = fs::read_to_string(&path).expect("wallet source must be readable");
            forbidden
                .iter()
                .filter(move |marker| contents.contains(**marker))
                .map(move |marker| format!("{} contains `{marker}`", path.display()))
        })
        .collect()
}

fn missing_source_markers(root: &Path, required: &[&str]) -> Vec<String> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    let contents = files
        .iter()
        .map(|path| fs::read_to_string(path).expect("wallet source must be readable"))
        .collect::<String>();
    required
        .iter()
        .filter(|marker| !contents.contains(**marker))
        .map(|marker| format!("wallet source lacks `{marker}`"))
        .collect()
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("wallet source directory must be readable") {
        let entry = entry.expect("wallet source directory entry must be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}
