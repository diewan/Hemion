//! Read-only projections for the Parwana conformance corpus and replay demonstration.

use crate::services::bundle_verifier::{ContextChoice, LocalVerificationError, import_and_verify};
use csv_sdk::accountability_verification::StageDisposition;

/// A published Parwana corpus entry. Expected results are contract metadata,
/// while actual results always come from the pinned SDK verifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixtureCase {
    pub id: &'static str,
    pub title: &'static str,
    pub expected: &'static str,
    pub explanation: &'static str,
}

/// The C-05 accountability-v0.1 corpus, in its published order.
pub const FIXTURE_CASES: [FixtureCase; 8] = [
    FixtureCase {
        id: "valid",
        title: "Valid deployment",
        expected: "Requirements met",
        explanation: "The exact mandate, intent, executor, time, evidence, receipt, and fresh replay status satisfy the selected context.",
    },
    FixtureCase {
        id: "expired",
        title: "Expired mandate",
        expected: "Requirements not met · MandateExpired",
        explanation: "Evaluation at or after the exclusive expiry boundary must fail closed.",
    },
    FixtureCase {
        id: "replayed",
        title: "Replayed mandate",
        expected: "Requirements not met · ReplayDetected",
        explanation: "The replay journal reports that this single-use mandate was already consumed.",
    },
    FixtureCase {
        id: "mutated-intent",
        title: "Mutated intent",
        expected: "Requirements not met · IntentMismatch",
        explanation: "A changed commit or environment no longer matches the approved intent.",
    },
    FixtureCase {
        id: "forged-source",
        title: "Forged source",
        expected: "Requirements not met · EvidenceAuthenticityRejected",
        explanation: "Evidence whose source authenticity is rejected cannot support the result.",
    },
    FixtureCase {
        id: "ambiguous-outcome",
        title: "Ambiguous outcome",
        expected: "Cannot be determined · OutcomeAmbiguous",
        explanation: "Uncertain execution is preserved; it is never converted to success or failure.",
    },
    FixtureCase {
        id: "selectively-disclosed",
        title: "Selective disclosure",
        expected: "Cannot be determined · SelectiveDisclosureLimitsEvaluation",
        explanation: "Withheld evidence limits evaluation without implying non-occurrence.",
    },
    FixtureCase {
        id: "missing-required-evidence",
        title: "Missing required evidence",
        expected: "Cannot be determined · RequiredEvidenceMissing",
        explanation: "Absent required evidence remains an explicit gap, not proof that an event did not happen.",
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comparison {
    pub expected: &'static str,
    pub actual: String,
    pub matches: bool,
}

fn fixture_case(case_id: &str) -> Option<&'static FixtureCase> {
    FIXTURE_CASES.iter().find(|case| case.id == case_id)
}

/// Verify one imported case and compare its actual disposition/reason codes
/// with the selected published corpus expectation.
pub fn run_case(
    case_id: &str,
    bundle: &[u8],
    context: ContextChoice,
) -> Result<Comparison, LocalVerificationError> {
    let case = fixture_case(case_id).ok_or(LocalVerificationError::UnsupportedBundleEncoding)?;
    let selected = context.name.clone();
    let result = import_and_verify(bundle, &[context], &selected)?;
    let disposition =
        crate::services::bundle_verifier::disposition_label(result.report.disposition);
    let reasons = result
        .report
        .stages
        .iter()
        .filter_map(|stage| match stage.disposition {
            StageDisposition::Pass => None,
            StageDisposition::Fail(reason)
            | StageDisposition::Indeterminate(reason)
            | StageDisposition::Unsupported(reason) => Some(reason),
        })
        .map(|reason| format!("{reason:?}"))
        .collect::<Vec<_>>();
    let actual = if reasons.is_empty() {
        disposition.to_owned()
    } else {
        format!("{disposition} · {}", reasons.join(", "))
    };
    let expected_parts = case.expected.split(" · ").collect::<Vec<_>>();
    let matches = actual.starts_with(expected_parts[0])
        && expected_parts
            .get(1)
            .is_none_or(|reason| actual.contains(reason));
    Ok(Comparison {
        expected: case.expected,
        actual,
        matches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_every_required_positive_and_adversarial_vector() {
        assert_eq!(FIXTURE_CASES.len(), 8);
        for id in [
            "valid",
            "expired",
            "replayed",
            "mutated-intent",
            "forged-source",
            "ambiguous-outcome",
            "selectively-disclosed",
            "missing-required-evidence",
        ] {
            assert!(FIXTURE_CASES.iter().any(|case| case.id == id));
        }
    }

    #[test]
    fn unknown_case_is_not_silently_mapped_to_a_published_vector() {
        assert!(fixture_case("invented").is_none());
    }
}
