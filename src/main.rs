use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use env_logger;
use log::{error, info};

use mjlog_parser::{parse_file, parse_stream, ParserOptions};

#[derive(Parser)]
#[command(name = "mjlog-parser")]
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();

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

        parse_file(&args.input, &output_path, &options)
            .with_context(|| {
                format!(
                    "Failed to parse mjlog from {:?} to {:?}",
                    args.input, output_path
                )
            })?;

        info!("Successfully parsed mjlog to: {:?}", output_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::NamedTempFile;

    #[test]
    fn test_help_command() {
        let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
            .arg("--help")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("mjlog-parser"));
        assert!(stdout.contains("A parser for Tenhou mjlog files"));
    }

    #[test]
    fn test_version_command() {
        let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
            .arg("--version")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
    }

    #[test]
    fn test_nonexistent_file() {
        let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
            .arg("nonexistent.xml.gz")
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
    }

    #[test]
    fn test_stream_mode_with_minimal_xml() {
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<mjloggm ver="2.3">
    <GO type="169" lobby="0"/>
    <UN n0="Player1" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,M,M,M"/>
</mjloggm>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, xml_content.as_bytes()).unwrap();

        let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
            .arg(temp_file.path())
            .arg("--stream")
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("mjlogVersion"));
        assert!(stdout.contains("players"));
    }
}