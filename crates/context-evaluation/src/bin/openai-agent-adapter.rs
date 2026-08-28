//! `OpenAI` production agent adapter for Impresari Context evaluation.

#![forbid(unsafe_code)]

use context_evaluation::provider_adapter::{Provider, run_stdio};

fn main() {
    if let Err(error) = run_stdio(Provider::OpenAi) {
        eprintln!("OpenAI agent adapter: {error}");
        std::process::exit(1);
    }
}
