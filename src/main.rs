use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info};

use tenhou_log_parser::{parse_file, parse_stream, ParserOptions};

#[derive(Parser)]
#[command(name = "tenhou-log-parser")]
#[command(about = "A parser for Tenhou mjlog files to JSON conversion")]
#[command(version)]
struct Args {
    /// Input mjlog file (.xml or .xml.gz)
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output JSON file path
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Force overwrite existing output file
    #[arg(short, long)]
    force: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Output to stdout instead of file
    #[arg(long)]
    stream: bool,

    /// JSON Schema file for validation
    #[arg(long, value_name = "FILE")]
    schema: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // Validate input file
    if !args.input.exists() {
        error!("Input file does not exist: {:?}", args.input);
        std::process::exit(1);
    }

    let options = ParserOptions {
        verbose: args.verbose,
        validate_schema: args.schema,
    };

    if args.stream {
        // Stream mode: output to stdout
        let file = std::fs::File::open(&args.input)
            .with_context(|| format!("Failed to open input file: {:?}", args.input))?;

        let reader: Box<dyn std::io::Read> = if args
            .input
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with("gz"))
            .unwrap_or(false)
        {
            Box::new(flate2::read::GzDecoder::new(file))
        } else {
            Box::new(file)
        };

        parse_stream(reader, std::io::stdout(), &options)
            .context("Failed to parse mjlog to stdout")?;
    } else {
        // File mode: output to file
        let output_path = match args.output {
            Some(path) => path,
            None => {
                let mut path = args.input.clone();
                path.set_extension("json");
                path
            }
        };

        // Check if output file exists and force flag
        if output_path.exists() && !args.force {
            error!(
                "Output file already exists: {:?}. Use --force to overwrite.",
                output_path
            );
            std::process::exit(1);
        }

        parse_file(&args.input, &output_path, &options).with_context(|| {
            format!(
                "Failed to parse mjlog from {:?} to {:?}",
                args.input, output_path
            )
        })?;

        info!("Successfully parsed mjlog to: {:?}", output_path);
    }

    Ok(())
}
