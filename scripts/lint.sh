#! /bin/bash
cd "$1"

cargo fmt

CLIPPY_FLAGS=(
    "-D" "warnings"                # Make all warnings deny
    "-W" "clippy::all"            # All default lints
    "-W" "clippy::cargo"          # Cargo-related lints
    "-D" "clippy::perf"           # Performance-related lints
    "-D" "clippy::complexity"     # Code complexity lints
    "-D" "clippy::style"          # Style lints
    # Exclude some overly strict lints
    "-A" "clippy::must_use_candidate"
    "-A" "clippy::missing_errors_doc"
    "-A" "clippy::module_name_repetitions"
    "-A" "clippy::multiple_crate_versions"
    "-A" "clippy::cargo_common_metadata"
    "-A" "clippy::negative_feature_names"
)

# Run clippy on all targets (including tests and examples)
cargo clippy --fix -- "${CLIPPY_FLAGS[@]}"

