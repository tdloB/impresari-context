// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Local command-line interface for Impresari Context."]

use std::io;

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let code = context_cli::execute(&arguments, &mut io::stdout(), &mut io::stderr());
    std::process::exit(code);
}
