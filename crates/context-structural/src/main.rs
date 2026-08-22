// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Capability-reduced structural parsing worker executable."]

fn main() {
    if context_structural::run_stdio().is_err() {
        std::process::exit(1);
    }
}
