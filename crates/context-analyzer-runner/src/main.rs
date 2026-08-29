// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Short-lived synthetic analyzer worker for ADR-0074 IAR-1A."]

fn main() {
    if context_analyzer_runner::run_worker_stdio().is_err() {
        std::process::exit(1);
    }
}
