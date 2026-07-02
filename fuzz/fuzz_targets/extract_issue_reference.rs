#![no_main]
use libfuzzer_sys::fuzz_target;
use merge_warden_core::checks::{extract_any_issue_reference, extract_closing_issue_reference};

// Invariants exercised:
//   1. Neither function panics on arbitrary UTF-8 input.
//   2. `extract_closing_issue_reference` returning `Some` implies
//      `extract_any_issue_reference` also returns `Some` (the any-variant
//      is strictly more permissive than the closing-only variant).
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let closing = extract_closing_issue_reference(s);
        let any = extract_any_issue_reference(s);

        // If a closing reference was found, the any-reference must also be present.
        if closing.is_some() {
            assert!(
                any.is_some(),
                "extract_any_issue_reference returned None but extract_closing returned {:?} \
                 for input: {:?}",
                closing,
                s
            );
        }
    }
});
