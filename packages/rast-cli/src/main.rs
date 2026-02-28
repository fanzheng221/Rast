use clap::{Parser, Subcommand};
use std::path::PathBuf;

use rast_cli::OutputFormat;

#[derive(Parser)]
#[command(name = "rast")]
#[command(about = "Rast - Pattern matching and code rewriting tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run(RunCommand),
    Scan(ScanCommand),
}

#[derive(Parser)]
#[command(about = "Apply a YAML rule to a single file")]
struct RunCommand {
    #[arg(value_name = "FILE")]
    file: PathBuf,

    #[arg(value_name = "RULE")]
    rule: String,

    #[arg(short, long, default_value = "json")]
    output: OutputFormat,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Parser)]
#[command(about = "Apply a YAML rule to all files in a directory")]
struct ScanCommand {
    #[arg(value_name = "DIR")]
    dir: PathBuf,

    #[arg(value_name = "RULE")]
    rule: String,

    #[arg(long)]
    dry_run: bool,

    #[arg(short, long, default_value = "json")]
    output: OutputFormat,

    #[arg(short = 'e', long, value_delimiter = ',')]
    extensions: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run(cmd) => run_command(cmd),
        Commands::Scan(cmd) => scan_command(cmd),
    }
}

fn run_command(cmd: RunCommand) {
    let result = rast_cli::run(&cmd.file, &cmd.rule, cmd.output, cmd.verbose)
        .expect("Failed to run command");

    print_output(&result, cmd.output);
}

fn scan_command(cmd: ScanCommand) {
    let result = rast_cli::scan(
        &cmd.dir,
        &cmd.rule,
        cmd.dry_run,
        cmd.output,
        cmd.extensions,
        cmd.verbose,
    )
    .expect("Failed to scan command");

    print_output(&result, cmd.output);
}

fn print_output(result: &str, format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{}", result),
        OutputFormat::Text => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(result) {
                print_text_output(&parsed);
            } else {
                println!("{}", result);
            }
        }
    }
}

fn print_text_output(parsed: &serde_json::Value) {
    if parsed.is_array() {
        for item in parsed.as_array().expect("array checked") {
            if let Some(path) = item.get("path").and_then(|v| v.as_str()) {
                println!("File: {}", path);
            }
            if let Some(matches) = item.get("matches").and_then(|v| v.as_u64()) {
                println!("  Matches: {}", matches);
            }
            if let Some(mods) = item.get("modifications").and_then(|v| v.as_u64()) {
                println!("  Modifications: {}", mods);
            }
        }
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(parsed).expect("failed to format json")
        );
    }
}
