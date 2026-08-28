//! Anthropic production agent adapter for Impresari Context evaluation.

#![forbid(unsafe_code)]

use context_evaluation::provider_adapter::{Provider, run_stdio, run_token_preflight_stdio};

fn main() {
    let result = if std::env::args().nth(1).as_deref() == Some("--count-tokens") {
        run_token_preflight_stdio(Provider::Anthropic)
    } else {
        run_stdio(Provider::Anthropic)
    };
    if let Err(error) = result {
        eprintln!("Anthropic agent adapter: {error}");
        std::process::exit(1);
    }
}
