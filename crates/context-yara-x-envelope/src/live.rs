// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "ADR-0102 synthetic-only live YARA-X envelope coordinator."]

fn main() {
    if context_yara_x_envelope::run_live_yara_x_stdio().is_err() {
        std::process::exit(1);
    }
}
