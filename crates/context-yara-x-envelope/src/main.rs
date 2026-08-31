// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Test-only ADR-0101 synthetic envelope coordinator."]

fn main() {
    if context_yara_x_envelope::run_coordinator_stdio().is_err() {
        std::process::exit(1);
    }
}
