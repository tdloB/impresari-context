//! Anthropic production agent adapter for Impresari Context evaluation.

#![forbid(unsafe_code)]

use context_evaluation::provider_adapter::{Provider, run_stdio};

fn main() {
    if let Err(error) = run_stdio(Provider::Anthropic) {
        eprintln!("Anthropic agent adapter: {error}");
        std::process::exit(1);
    }
}
