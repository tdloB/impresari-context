# Coverage-guided fuzzing

This directory contains isolated `cargo-fuzz` targets. It is not part of the
release workspace or release artifacts.

The `context_packet` target exercises untrusted JSON deserialization, packet
integrity validation, and canonical serialization. A packet accepted by the
validator must round-trip without changing and without panicking.

Run locally with the pinned tool version and a nightly Rust toolchain:

```console
cargo install cargo-fuzz --version 0.13.2 --locked
cargo +nightly fuzz run context_packet -- -max_total_time=60 -max_len=1048576
```

CI runs a short bounded campaign for changes and a longer scheduled campaign.
Crashing inputs are retained as short-lived workflow artifacts for triage.
