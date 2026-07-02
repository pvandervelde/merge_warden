#![no_main]
use libfuzzer_sys::fuzz_target;
use merge_warden_core::checks::diagnose_pr_title;

// Invariant: `diagnose_pr_title` must never panic on any byte sequence that
// is valid UTF-8.  An Err / invalid result is acceptable; a panic is not.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = diagnose_pr_title(s);
    }
});
