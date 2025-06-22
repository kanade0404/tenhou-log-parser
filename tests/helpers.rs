use std::path::PathBuf;

/// Returns the path to a test data file located in the `tests/data` directory relative to the project root.
///
/// # Parameters
/// - `filename`: The name of the test data file.
///
/// # Returns
/// The full path to the specified test data file as a `PathBuf`.
pub fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(filename)
}

/// Returns a minimal mjlog XML string representing a basic Mahjong game log scenario.
///
/// The XML includes essential elements for initializing a game with four players and a simple game flow. Useful for testing parsers or components that require a valid mjlog input.
pub fn minimal_mjlog() -> String {
    r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3" shuffle="mt19937ar-sha512-n288-base64">
    <GO type="169" lobby="0"/>
    <UN n0="Player1" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,M,M,M"/>
    <TAIKYOKU oya="0"/>
    <INIT seed="0,0,0,1,2,3" ten="250,250,250,250" oya="0" hai0="0,4,8,12,16,20,24,28,32,36,40,44,48" hai1="1,5,9,13,17,21,25,29,33,37,41,45,49" hai2="2,6,10,14,18,22,26,30,34,38,42,46,50" hai3="3,7,11,15,19,23,27,31,35,39,43,47,51"/>
    <T52/>
    <D0/>
    <RYUUKYOKU ba="0,0" sc="250,0,250,0,250,0,250,0" type="nm"/>
</mjloggm>"#.to_string()
}

/// Returns a complete mjlog XML string representing a detailed Mahjong game log for testing purposes.
///
/// The returned XML includes player information, initial game state, tile draws and discards, dora indicators, reach declarations, and an agari (win) event. This fixture is suitable for tests requiring a comprehensive mjlog scenario.
pub fn complete_mjlog() -> String {
    r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3" shuffle="mt19937ar-sha512-n288-base64">
    <GO type="169" lobby="0"/>
    <UN n0="TestPlayer1" n1="TestPlayer2" n2="TestPlayer3" n3="TestPlayer4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,F,M,F"/>
    <TAIKYOKU oya="0"/>
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0,4,8,12,16,20,24,28,32,36,40,44,48" hai1="1,5,9,13,17,21,25,29,33,37,41,45,49" hai2="2,6,10,14,18,22,26,30,34,38,42,46,50" hai3="3,7,11,15,19,23,27,31,35,39,43,47,51"/>
    <T53/>
    <D0/>
    <U54/>
    <E1/>
    <V55/>
    <F2/>
    <W56/>
    <G3/>
    <DORA hai="57"/>
    <REACH who="0" step="1" ten="240,250,250,250"/>
    <T58/>
    <D58/>
    <REACH who="0" step="2"/>
    <AGARI ba="0,10" hai="4,8,12,16,20,24,28,32,36,40,44,48,59" machi="59" ten="30,1000,0" yaku="1,1" doraHai="57" who="0" fromWho="0" sc="240,1010,250,-250,250,-250,250,-250"/>
</mjloggm>"#.to_string()
}
