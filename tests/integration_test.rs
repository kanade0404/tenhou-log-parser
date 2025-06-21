use mjlog_parser::{parse_mjlog, ParserOutput};
use std::io::Cursor;
use std::process::Command;
use tempfile::NamedTempFile;

mod helpers;
use helpers::{complete_mjlog, minimal_mjlog, test_data_path};

#[test]
fn test_parse_minimal_mjlog() {
    let mjlog_content = minimal_mjlog();
    let cursor = Cursor::new(mjlog_content.as_bytes());
    let result = parse_mjlog(cursor);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.mjlog_version, "2.3");
    assert_eq!(output.players.len(), 4);
    assert_eq!(output.players[0].player_id, "Player1");
    assert_eq!(output.players[0].seat, 0);
}

#[test]
fn test_parse_complete_mjlog() {
    let mjlog_content = complete_mjlog();
    let cursor = Cursor::new(mjlog_content.as_bytes());
    let result = parse_mjlog(cursor);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.mjlog_version, "2.3");
    assert_eq!(output.players.len(), 4);
    assert_eq!(output.rounds.len(), 1);

    // Check that events were parsed
    let round = &output.rounds[0];
    assert!(!round.events.is_empty());
}

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("mjlog-parser"));
    assert!(stdout.contains("A parser for Tenhou mjlog files"));
}

#[test]
fn test_cli_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

#[test]
fn test_cli_nonexistent_file() {
    let output = Command::new("cargo")
        .args(&["run", "--", "nonexistent.xml.gz"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_cli_stream_mode() {
    let xml_content = minimal_mjlog();
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, xml_content.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", temp_file.path().to_str().unwrap(), "--stream"])
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

#[test]
fn test_json_serialization() {
    let mjlog_content = minimal_mjlog();
    let cursor = Cursor::new(mjlog_content.as_bytes());
    let result = parse_mjlog(cursor).unwrap();

    // Test JSON serialization
    let json_result = serde_json::to_string_pretty(&result);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    assert!(json_str.contains("mjlogVersion"));
    assert!(json_str.contains("players"));
    assert!(json_str.contains("rounds"));

    // Test deserialization back
    let parsed_back: Result<ParserOutput, _> = serde_json::from_str(&json_str);
    assert!(parsed_back.is_ok());
}

#[test]
fn test_sample_xml_file_parsing() {
    let sample_path = test_data_path("sample.xml");
    
    // Check if sample file exists
    if sample_path.exists() {
        let content = std::fs::read_to_string(&sample_path).unwrap();
        let cursor = Cursor::new(content.as_bytes());
        let result = parse_mjlog(cursor);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.mjlog_version, "2.3");
        assert_eq!(output.players.len(), 4);
    } else {
        // Skip test if sample file doesn't exist
        eprintln!("Sample file {:?} not found, skipping test", sample_path);
    }
}
