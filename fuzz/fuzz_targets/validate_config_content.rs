#![no_main]
use libfuzzer_sys::fuzz_target;
use merge_warden_core::config::validate_config_content;

// Invariant: `validate_config_content` must never panic on any UTF-8 string,
// regardless of whether it is valid TOML or a syntactically valid config.
// Returning a validation error is acceptable; a panic is not.
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = validate_config_content(s);
    }
});
