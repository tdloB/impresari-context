// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Closed Impresari-owned original-synthetic YARA-X-shaped emitter."]

use std::io::Write as _;

fn main() {
    let mut arguments = std::env::args();
    let _program = arguments.next();
    let Some(case_id) = arguments.next() else {
        std::process::exit(2);
    };
    if arguments.next().is_some() {
        std::process::exit(2);
    }
    let Ok(record) = context_yara_x_envelope::synthetic_record(&case_id) else {
        std::process::exit(2);
    };
    if std::io::stdout().lock().write_all(record).is_err() {
        std::process::exit(1);
    }
}
