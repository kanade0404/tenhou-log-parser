use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_e2e_sample_xml() {
    let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
        .args(&["tests/data/sample.xml", "--stream"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify JSON structure
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse JSON output");

    // Basic structure checks
    assert!(parsed.get("mjlogVersion").is_some());
    assert!(parsed.get("gameId").is_some());
    assert!(parsed.get("rules").is_some());
    assert!(parsed.get("players").is_some());
    assert!(parsed.get("rounds").is_some());

    // Check players array
    let players = parsed["players"].as_array().unwrap();
    assert_eq!(players.len(), 4);

    // Check first player
    let player0 = &players[0];
    assert_eq!(player0["seat"], 0);
    assert_eq!(player0["playerId"], "テストユーザー1");
    assert_eq!(player0["rank"], 5);
    assert_eq!(player0["rate"], 1800);
    assert_eq!(player0["gender"], "M");

    // Check rounds
    let rounds = parsed["rounds"].as_array().unwrap();
    assert_eq!(rounds.len(), 1);

    let round = &rounds[0];
    assert!(round.get("init").is_some());
    assert!(round.get("events").is_some());

    // Check initial hands
    let init = &round["init"];
    let initial_hands = init["initialHands"].as_array().unwrap();
    assert_eq!(initial_hands.len(), 4);

    // Each player should have 13 tiles initially
    for hand in initial_hands {
        assert_eq!(hand.as_array().unwrap().len(), 13);
    }

    // Check events
    let events = round["events"].as_array().unwrap();
    assert!(!events.is_empty());

    // Should have dora, reach, and agari events
    let event_types: Vec<&str> = events.iter().map(|e| e["type"].as_str().unwrap()).collect();

    assert!(event_types.contains(&"dora"));
    assert!(event_types.contains(&"reach"));
    assert!(event_types.contains(&"agari"));
}

#[test]
fn test_e2e_file_output() {
    let temp_output = NamedTempFile::new().unwrap();
    let output_path = temp_output.path().to_str().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
        .args(&["tests/data/sample.xml", "-o", output_path, "-f"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read the output file
    let file_content = std::fs::read_to_string(output_path).unwrap();

    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&file_content).expect("Failed to parse JSON from output file");

    assert!(parsed.get("mjlogVersion").is_some());
    assert_eq!(parsed["players"].as_array().unwrap().len(), 4);
}

#[test]
fn test_e2e_verbose_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
        .args(&["tests/data/sample.xml", "--stream", "-v"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should still produce valid JSON despite verbose logging
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse JSON output in verbose mode");

    assert!(parsed.get("mjlogVersion").is_some());
}

#[test]
fn test_e2e_nonexistent_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
        .args(&["nonexistent.xml"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Input file does not exist"));
}

#[test]
fn test_e2e_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
        .args(&["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("mjlog-parser"));
    assert!(stdout.contains("A parser for Tenhou mjlog files"));
    assert!(stdout.contains("--stream"));
    assert!(stdout.contains("--verbose"));
}

#[test]
fn test_e2e_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_mjlog-parser"))
        .args(&["--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("0.1.0"));
}
