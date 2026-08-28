//! Command-line interface for developer agent-context evaluation studies.

#![forbid(unsafe_code)]

//! Command-line interface for bounded agent-context A/B/A studies.

use context_evaluation::agent_eval::{
    default_summary_paths, load_records, load_spec, run_study, summarize, validate_records,
    write_json, write_markdown,
};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map_or("help", String::as_str);
    match command {
        "validate-spec" => {
            require_arg_count(&args, 2, "validate-spec <study.json>")?;
            let spec = required_path(&args, 1, "study specification")?;
            load_spec(&spec)?;
            println!("valid: {}", spec.display());
        }
        "run" => {
            let spec_path = required_path(&args, 1, "study specification")?;
            let output_dir = required_path(&args, 2, "output directory")?;
            let explicit_consent = args
                .get(3)
                .is_some_and(|value| value == "--allow-adapter-execution");
            if args.len() != 4 || !explicit_consent {
                return Err(
                    "run requires exactly: <study.json> <output-dir> --allow-adapter-execution"
                        .to_owned(),
                );
            }
            let spec = load_spec(&spec_path)?;
            let records = run_study(&spec, &output_dir, explicit_consent)?;
            let summary = summarize(&spec, &records)?;
            let (json_path, markdown_path) = default_summary_paths(&output_dir);
            write_json(&json_path, &summary)?;
            write_markdown(&markdown_path, &summary)?;
            println!(
                "completed {} runs in {}",
                records.len(),
                output_dir.display()
            );
        }
        "validate-runs" => {
            require_arg_count(&args, 3, "validate-runs <study.json> <run-dir>")?;
            let spec_path = required_path(&args, 1, "study specification")?;
            let input_dir = required_path(&args, 2, "run directory")?;
            let spec = load_spec(&spec_path)?;
            let records = load_records(&input_dir)?;
            validate_records(&spec, &records)?;
            println!("valid: {} run records", records.len());
        }
        "summarize" => {
            if !(3..=4).contains(&args.len()) {
                return Err("usage: summarize <study.json> <run-dir> [output-dir]".to_owned());
            }
            let spec_path = required_path(&args, 1, "study specification")?;
            let input_dir = required_path(&args, 2, "run directory")?;
            let output_dir = args.get(3).map_or_else(|| input_dir.clone(), PathBuf::from);
            let spec = load_spec(&spec_path)?;
            let records = load_records(&input_dir)?;
            let summary = summarize(&spec, &records)?;
            let (json_path, markdown_path) = default_summary_paths(&output_dir);
            write_json(&json_path, &summary)?;
            write_markdown(&markdown_path, &summary)?;
            println!(
                "wrote {} and {}",
                json_path.display(),
                markdown_path.display()
            );
        }
        "help" | "--help" | "-h" => print_help(),
        other => return Err(format!("unknown command {other:?}")),
    }
    Ok(())
}

fn require_arg_count(args: &[String], expected: usize, usage: &str) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!("usage: {usage}"))
    }
}

fn required_path(args: &[String], index: usize, description: &str) -> Result<PathBuf, String> {
    args.get(index)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing {description}"))
}

fn print_help() {
    println!(
        "impresari-context-agent-eval\n\n\
         Commands:\n  \
         validate-spec <study.json>\n  \
         run <study.json> <output-dir> --allow-adapter-execution\n  \
         validate-runs <study.json> <run-dir>\n  \
         summarize <study.json> <run-dir> [output-dir]"
    );
}
