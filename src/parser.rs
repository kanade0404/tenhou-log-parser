use std::io::{Read, Write};
use std::path::Path;

use encoding_rs::SHIFT_JIS;
use flate2::read::GzDecoder;
use log::{debug, info};
use quick_xml::events::Event as XmlEvent;
use quick_xml::Reader;

use crate::error::{ParserError, Result};
use crate::models::{Event, Init, ParserOutput, Player, Round, Rules, RyuukyokuReason, Yaku};
use crate::tile::{parse_tile_list, tile_id_to_string};

#[derive(Debug, Clone, Default)]
pub struct ParserOptions {
    pub verbose: bool,
    pub validate_schema: Option<std::path::PathBuf>,
}

/// Parse mjlog file and write JSON to output
pub fn parse_file(input_path: &Path, output_path: &Path, options: &ParserOptions) -> Result<()> {
    info!("Parsing mjlog file: {:?}", input_path);

    let file = std::fs::File::open(input_path).map_err(ParserError::Io)?;

    let reader: Box<dyn Read> = if input_path.extension().and_then(|s| s.to_str()) == Some("gz") {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let output_file = std::fs::File::create(output_path).map_err(ParserError::Io)?;

    parse_stream(reader, output_file, options)?;

    info!("Successfully parsed mjlog and wrote to: {:?}", output_path);
    Ok(())
}

/// Parse mjlog from reader and write JSON to writer
pub fn parse_stream<R: Read, W: Write>(
    reader: R,
    mut writer: W,
    _options: &ParserOptions,
) -> Result<()> {
    let parser_output = parse_mjlog(reader)?;

    serde_json::to_writer_pretty(&mut writer, &parser_output)
        .map_err(|e| ParserError::Io(std::io::Error::other(e)))?;

    Ok(())
}

/// Maximum file size limit (100MB) to prevent memory exhaustion
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Parse mjlog from reader and return ParserOutput
pub fn parse_mjlog<R: Read>(reader: R) -> Result<ParserOutput> {
    let reader = std::io::BufReader::new(reader);
    let mut buf = Vec::new();
    
    // Read with size limit to prevent memory exhaustion
    let mut limited_reader = reader.take(MAX_FILE_SIZE as u64);
    limited_reader.read_to_end(&mut buf)?;
    
    // Check if we hit the size limit
    if buf.len() >= MAX_FILE_SIZE {
        return Err(ParserError::parse(
            format!("File too large (>{} bytes). Maximum allowed size is {} bytes", 
                    buf.len(), MAX_FILE_SIZE),
            "file size validation"
        ));
    }

    // Convert from Shift_JIS to UTF-8
    let (content, encoding_used, had_errors) = SHIFT_JIS.decode(&buf);
    if had_errors {
        // Log warning but continue processing
        // Only fail if the encoding is completely wrong or data is severely corrupted
        if encoding_used != encoding_rs::UTF_8 && content.is_empty() {
            return Err(ParserError::encoding(
                "Critical encoding error: Unable to decode any content from Shift_JIS"
            ));
        }
        // For partial errors, log and continue
        debug!("Encoding errors detected during Shift_JIS to UTF-8 conversion, but continuing");
    }

    let mut xml_reader = Reader::from_str(&content);
    xml_reader.trim_text(true);

    let mut parser = MjlogParser::new();
    parser.parse(&mut xml_reader)?;

    Ok(parser.into_output())
}

struct MjlogParser {
    mjlog_version: String,
    game_id: String,
    rules: Option<Rules>,
    players: Vec<Player>,
    rounds: Vec<Round>,
    current_round: Option<Round>,
}

impl MjlogParser {
    fn new() -> Self {
        Self {
            mjlog_version: String::new(),
            game_id: uuid::Uuid::new_v4().to_string(),
            rules: None,
            players: Vec::new(),
            rounds: Vec::new(),
            current_round: None,
        }
    }

    fn parse<R: std::io::BufRead>(&mut self, reader: &mut Reader<R>) -> Result<()> {
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                XmlEvent::Start(ref e) | XmlEvent::Empty(ref e) => {
                    let tag_name = e.name();
                    let tag_bytes = tag_name.as_ref();
                    match tag_bytes {
                        b"mjloggm" => self.parse_mjloggm(e)?,
                        b"GO" => self.parse_go(e)?,
                        b"UN" => self.parse_un(e)?,
                        b"TAIKYOKU" => self.parse_taikyoku(e)?,
                        b"INIT" => self.parse_init(e)?,
                        b"N" => self.parse_naki(e)?,
                        b"DORA" => self.parse_dora(e)?,
                        b"REACH" => self.parse_reach(e)?,
                        b"AGARI" => self.parse_agari(e)?,
                        b"RYUUKYOKU" => self.parse_ryuukyoku(e)?,
                        _ => {
                            // Check if it's a draw or discard tag
                            if !tag_bytes.is_empty() {
                                let first_byte = tag_bytes[0];
                                match first_byte {
                                    b'T' | b'U' | b'V' | b'W' => self.parse_draw(e)?,
                                    b'D' | b'E' | b'F' | b'G' => self.parse_discard(e)?,
                                    _ => {
                                        debug!("Unknown tag: {:?}", std::str::from_utf8(tag_bytes));
                                    }
                                }
                            }
                        }
                    }
                }
                XmlEvent::End(_) => {}
                XmlEvent::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        // Finish current round if any
        if let Some(round) = self.current_round.take() {
            self.rounds.push(round);
        }

        Ok(())
    }

    fn parse_mjloggm(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            if attr.key.as_ref() == b"ver" {
                self.mjlog_version = std::str::from_utf8(&attr.value)?.to_string();
            }
        }
        Ok(())
    }

    fn parse_go(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let mut type_flags = 0;
        let mut lobby_id = None;

        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            match attr.key.as_ref() {
                b"type" => {
                    type_flags = std::str::from_utf8(&attr.value)?.parse()?;
                }
                b"lobby" => {
                    let value: u32 = std::str::from_utf8(&attr.value)?.parse()?;
                    lobby_id = if value == 0 { None } else { Some(value) };
                }
                _ => {}
            }
        }

        self.rules = Some(Rules {
            type_flags,
            lobby_id,
        });
        Ok(())
    }

    fn parse_un(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let mut names = vec![String::new(); 4];
        let mut dans = [0u32; 4];
        let mut rates = [0u32; 4];
        let mut genders = vec![String::new(); 4];

        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            let key = std::str::from_utf8(attr.key.as_ref())?;
            let value = std::str::from_utf8(&attr.value)?;

            match key {
                "n0" | "n1" | "n2" | "n3" => {
                    let seat = key.chars().last().unwrap().to_digit(10).unwrap() as usize;
                    names[seat] = percent_decode(value);
                }
                "dan" => {
                    let parts: Vec<&str> = value.split(',').collect();
                    for (i, &part) in parts.iter().enumerate().take(4) {
                        dans[i] = part.parse()?;
                    }
                }
                "rate" => {
                    let parts: Vec<&str> = value.split(',').collect();
                    for (i, &part) in parts.iter().enumerate().take(4) {
                        rates[i] = part.parse()?;
                    }
                }
                "sx" => {
                    let parts: Vec<&str> = value.split(',').collect();
                    for (i, &part) in parts.iter().enumerate().take(4) {
                        genders[i] = part.to_string();
                    }
                }
                _ => {}
            }
        }

        for i in 0..4 {
            self.players.push(Player {
                seat: i as u8,
                player_id: names[i].clone(),
                rank: dans[i],
                rate: rates[i],
                gender: genders[i].clone(),
            });
        }

        Ok(())
    }

    fn parse_taikyoku(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            if attr.key.as_ref() == b"oya" {
                let _oya: u8 = std::str::from_utf8(&attr.value)?.parse()?;
                // oya (dealer) information can be stored if needed
            }
        }
        Ok(())
    }

    fn parse_init(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let mut seed = String::new();
        let mut ten = String::new();
        let mut oya = 0u8;
        let mut hands = vec![String::new(); 4];

        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            let key = std::str::from_utf8(attr.key.as_ref())?;
            let value = std::str::from_utf8(&attr.value)?;

            match key {
                "seed" => seed = value.to_string(),
                "ten" => ten = value.to_string(),
                "oya" => oya = value.parse()?,
                "hai0" | "hai1" | "hai2" | "hai3" => {
                    let seat = key.chars().last().unwrap().to_digit(10).unwrap() as usize;
                    hands[seat] = value.to_string();
                }
                _ => {}
            }
        }

        // Parse seed: "局順,本場,供託,サイコロ1,サイコロ2,ドラ表示牌"
        let seed_parts: Vec<&str> = seed.split(',').collect();
        if seed_parts.len() < 6 {
            return Err(ParserError::invalid_format("Invalid seed format"));
        }

        let round_number: u32 = seed_parts[0].parse()?;
        let honba: u32 = seed_parts[1].parse()?;
        let kyoutaku: u32 = seed_parts[2].parse()?;
        let dice1: u32 = seed_parts[3].parse()?;
        let dice2: u32 = seed_parts[4].parse()?;
        let dora_indicator: u32 = seed_parts[5].parse()?;

        // Parse ten (scores)
        let ten_parts: Vec<&str> = ten.split(',').collect();
        if ten_parts.len() < 4 {
            return Err(ParserError::invalid_format("Invalid ten format"));
        }

        let initial_scores = [
            ten_parts[0].parse()?,
            ten_parts[1].parse()?,
            ten_parts[2].parse()?,
            ten_parts[3].parse()?,
        ];

        // Parse initial hands
        let mut initial_hands = Vec::new();
        for hand in &hands {
            let tiles = parse_tile_list(hand)?;
            initial_hands.push(tiles);
        }

        let init = Init {
            round_number,
            honba,
            kyoutaku,
            dice: [dice1, dice2],
            dora_indicator,
            initial_scores,
            initial_hands,
        };

        // Finish previous round if any
        if let Some(round) = self.current_round.take() {
            self.rounds.push(round);
        }

        let round_id = format!("Round {}", self.rounds.len() + 1);
        self.current_round = Some(Round {
            round_id,
            dealer_seat: oya,
            init,
            events: Vec::new(),
        });

        Ok(())
    }

    fn parse_draw(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let name = element.name();
        let tag_name = std::str::from_utf8(name.as_ref())?;
        let seat = match tag_name.chars().next().unwrap_or('\0') {
            'T' => 0,
            'U' => 1,
            'V' => 2,
            'W' => 3,
            _ => return Err(ParserError::invalid_format("Invalid draw tag")),
        };

        // Parse tile ID from element content/attributes
        if let Some(round) = &mut self.current_round {
            let mut tile_id = None;

            // Try to get tile ID from attributes first
            for attr in element.attributes() {
                let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
                if !attr.key.as_ref().is_empty() {
                    continue;
                }
                tile_id = Some(std::str::from_utf8(&attr.value)?.parse()?);
                break;
            }

            // If no attribute, try to parse from tag name (e.g., T52 -> 52)
            if tile_id.is_none() {
                let name = element.name();
                let tag_name = std::str::from_utf8(name.as_ref())?;
                if tag_name.len() > 1 {
                    if let Ok(id) = tag_name[1..].parse() {
                        tile_id = Some(id);
                    }
                }
            }

            if let Some(id) = tile_id {
                let tile = tile_id_to_string(id);
                round.events.push(Event::Draw { seat, tile });
            }
        }

        Ok(())
    }

    fn parse_discard(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let name = element.name();
        let tag_name = std::str::from_utf8(name.as_ref())?;
        let seat = match tag_name.chars().next().unwrap_or('\0') {
            'D' => 0,
            'E' => 1,
            'F' => 2,
            'G' => 3,
            _ => return Err(ParserError::invalid_format("Invalid discard tag")),
        };

        if let Some(round) = &mut self.current_round {
            let mut tile_id = None;

            // Try to get tile ID from attributes first
            for attr in element.attributes() {
                let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
                if !attr.key.as_ref().is_empty() {
                    continue;
                }
                tile_id = Some(std::str::from_utf8(&attr.value)?.parse()?);
                break;
            }

            // If no attribute, try to parse from tag name (e.g., D52 -> 52)
            if tile_id.is_none() {
                let name = element.name();
                let tag_name = std::str::from_utf8(name.as_ref())?;
                if tag_name.len() > 1 {
                    if let Ok(id) = tag_name[1..].parse() {
                        tile_id = Some(id);
                    }
                }
            }

            if let Some(id) = tile_id {
                let tile = tile_id_to_string(id);
                round.events.push(Event::Discard {
                    seat,
                    tile,
                    is_riichi: false, // TODO: Detect riichi discard
                });
            }
        }

        Ok(())
    }

    fn parse_naki(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let mut who = 0u8;
        let mut _meld = String::new();

        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            match attr.key.as_ref() {
                b"who" => who = std::str::from_utf8(&attr.value)?.parse()?,
                b"meld" => _meld = std::str::from_utf8(&attr.value)?.to_string(),
                _ => {}
            }
        }

        // TODO: Parse meld data to determine chi/pon/kan type and tiles
        // For now, create a generic pon event
        if let Some(round) = &mut self.current_round {
            let tiles = ["1m".to_string(), "1m".to_string(), "1m".to_string()];
            round.events.push(Event::Pon {
                who,
                tiles,
                from: 0, // TODO: Determine from meld data
            });
        }

        Ok(())
    }

    fn parse_dora(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            if attr.key.as_ref() == b"hai" {
                let tile_id: u32 = std::str::from_utf8(&attr.value)?.parse()?;
                let indicator = tile_id_to_string(tile_id);
                if let Some(round) = &mut self.current_round {
                    round.events.push(Event::Dora { indicator });
                }
            }
        }
        Ok(())
    }

    fn parse_reach(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let mut who = 0u8;
        let mut step = 1u8;
        let mut scores = [0i32; 4];

        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            match attr.key.as_ref() {
                b"who" => who = std::str::from_utf8(&attr.value)?.parse()?,
                b"step" => step = std::str::from_utf8(&attr.value)?.parse()?,
                b"ten" => {
                    let ten_str = std::str::from_utf8(&attr.value)?;
                    let parts: Vec<&str> = ten_str.split(',').collect();
                    for (i, &part) in parts.iter().enumerate().take(4) {
                        scores[i] = part.parse()?;
                    }
                }
                _ => {}
            }
        }

        if let Some(round) = &mut self.current_round {
            round.events.push(Event::Reach { who, step, scores });
        }

        Ok(())
    }

    fn parse_agari(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let mut who = 0u8;
        let mut from = 0u8;
        let mut han = 0u32;
        let mut fu = 0u32;
        let mut yakus = Vec::new();
        let dora_count = 0u32;
        let mut scores = [0i32; 4];

        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            match attr.key.as_ref() {
                b"who" => who = std::str::from_utf8(&attr.value)?.parse()?,
                b"fromWho" => from = std::str::from_utf8(&attr.value)?.parse()?,
                b"ten" => {
                    let ten_str = std::str::from_utf8(&attr.value)?;
                    let parts: Vec<&str> = ten_str.split(',').collect();
                    if parts.len() >= 3 {
                        fu = parts[0].parse()?;
                        let _score = parts[1].parse::<i32>()?; // Total score
                        han = parts[2].parse()?;
                    }
                }
                b"yaku" => {
                    // TODO: Parse yaku list
                    yakus.push(Yaku {
                        name: "Unknown".to_string(),
                        value: 1,
                    });
                }
                b"sc" => {
                    let sc_str = std::str::from_utf8(&attr.value)?;
                    let parts: Vec<&str> = sc_str.split(',').collect();
                    for (i, chunk) in parts.chunks(2).enumerate().take(4) {
                        if chunk.len() >= 2 {
                            let _before: i32 = chunk[0].parse()?;
                            let change: i32 = chunk[1].parse()?;
                            scores[i] = change;
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(round) = &mut self.current_round {
            round.events.push(Event::Agari {
                who,
                from,
                han,
                fu,
                yakus,
                dora_count,
                scores,
            });
        }

        Ok(())
    }

    fn parse_ryuukyoku(&mut self, element: &quick_xml::events::BytesStart) -> Result<()> {
        let mut reason = RyuukyokuReason::Normal;
        let mut scores = [0i32; 4];

        for attr in element.attributes() {
            let attr = attr.map_err(|e| ParserError::Attr(e.to_string()))?;
            match attr.key.as_ref() {
                b"type" => {
                    let type_str = std::str::from_utf8(&attr.value)?;
                    reason = match type_str {
                        "nm" => RyuukyokuReason::Normal,
                        "yao9" => RyuukyokuReason::Yao9,
                        "kaze4" => RyuukyokuReason::Kaze4,
                        "reach4" => RyuukyokuReason::Reach4,
                        "ron3" => RyuukyokuReason::Ron3,
                        "kan4" => RyuukyokuReason::Kan4,
                        _ => RyuukyokuReason::Normal,
                    };
                }
                b"sc" => {
                    let sc_str = std::str::from_utf8(&attr.value)?;
                    let parts: Vec<&str> = sc_str.split(',').collect();
                    for (i, chunk) in parts.chunks(2).enumerate().take(4) {
                        if chunk.len() >= 2 {
                            let _before: i32 = chunk[0].parse()?;
                            let change: i32 = chunk[1].parse()?;
                            scores[i] = change;
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(round) = &mut self.current_round {
            round.events.push(Event::Ryuukyoku { reason, scores });
        }

        Ok(())
    }

    fn into_output(self) -> ParserOutput {
        ParserOutput {
            mjlog_version: self.mjlog_version,
            game_id: self.game_id,
            rules: self.rules.unwrap_or(Rules {
                type_flags: 0,
                lobby_id: None,
            }),
            players: self.players,
            rounds: self.rounds,
        }
    }
}

// Helper function to decode percent-encoded strings
fn percent_decode(input: &str) -> String {
    percent_encoding::percent_decode_str(input)
        .decode_utf8_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Cursor;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_minimal_mjlog() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <GO type="169" lobby="0"/>
    <UN n0="Player1" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,M,M,M"/>
    <TAIKYOKU oya="0"/>
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0,4,8,12" hai1="1,5,9,13" hai2="2,6,10,14" hai3="3,7,11,15"/>
    <RYUUKYOKU ba="0,0" sc="250,0,250,0,250,0,250,0" type="nm"/>
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let result = parse_mjlog(cursor);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.mjlog_version, "2.3");
        assert_eq!(output.players.len(), 4);
        assert_eq!(output.rounds.len(), 1);
    }

    #[test]
    fn test_parse_with_gzip() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <GO type="169" lobby="0"/>
    <UN n0="Player1" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,M,M,M"/>
</mjloggm>"#;

        // Create a temporary gzipped file
        let mut temp_file = NamedTempFile::with_suffix(".mjlog").unwrap();

        // Write gzipped content
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(mjlog_content.as_bytes()).unwrap();
        let gzipped_data = encoder.finish().unwrap();
        temp_file.write_all(&gzipped_data).unwrap();
        temp_file.flush().unwrap();

        // Test parse_file with gzipped input
        let output_file = NamedTempFile::with_suffix(".json").unwrap();
        let options = ParserOptions::default();

        let result = parse_file(temp_file.path(), output_file.path(), &options);
        assert!(result.is_ok());

        // Verify output file was created and contains valid JSON
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.get("mjlogVersion").is_some());
    }

    #[test]
    fn test_parse_stream() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <GO type="169" lobby="0"/>
    <UN n0="Player1" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,M,M,M"/>
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let mut output = Vec::new();
        let options = ParserOptions::default();

        let result = parse_stream(cursor, &mut output, &options);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output_str).unwrap();
        assert!(parsed.get("mjlogVersion").is_some());
    }

    #[test]
    fn test_parse_with_encoding_errors() {
        // Create content with invalid Shift_JIS sequence
        let mut bad_content =
            b"<?xml version=\"1.0\" encoding=\"Shift_JIS\"?>\n<mjloggm ver=\"2.3\">\n".to_vec();
        bad_content.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid Shift_JIS
        bad_content.extend_from_slice(b"\n</mjloggm>");

        let cursor = Cursor::new(bad_content);
        let result = parse_mjlog(cursor);
        // Should still succeed but with encoding warnings
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_complex_mjlog() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <GO type="169" lobby="1"/>
    <UN n0="%E3%83%86%E3%82%B9%E3%83%88" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,F,M,F"/>
    <TAIKYOKU oya="0"/>
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0,4,8,12,16,20,24,28,32,36,40,44,48" hai1="1,5,9,13,17,21,25,29,33,37,41,45,49" hai2="2,6,10,14,18,22,26,30,34,38,42,46,50" hai3="3,7,11,15,19,23,27,31,35,39,43,47,51"/>
    <T52/>
    <D0/>
    <U53/>
    <E1/>
    <V54/>
    <F2/>
    <W55/>
    <G3/>
    <DORA hai="56"/>
    <REACH who="0" step="1" ten="240,250,250,250"/>
    <T57/>
    <D57/>
    <REACH who="0" step="2"/>
    <N who="1" m="12345"/>
    <AGARI who="0" fromWho="0" ten="30,1000,0" yaku="1,1" sc="240,1010,250,-250,250,-250,250,-250"/>
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let result = parse_mjlog(cursor);
        match &result {
            Ok(_) => {}
            Err(e) => panic!("Parse failed with error: {}", e),
        }
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.mjlog_version, "2.3");
        assert_eq!(output.players.len(), 4);
        assert_eq!(output.rules.lobby_id, Some(1)); // Non-zero lobby

        // Check that percent-encoded name was decoded
        assert_eq!(output.players[0].player_id, "テスト");

        // Check events were parsed
        let round = &output.rounds[0];
        assert!(!round.events.is_empty());

        // Check specific events exist
        let event_types: Vec<&str> = round
            .events
            .iter()
            .filter_map(|e| match e {
                Event::Draw { .. } => Some("draw"),
                Event::Discard { .. } => Some("discard"),
                Event::Dora { .. } => Some("dora"),
                Event::Reach { .. } => Some("reach"),
                Event::Pon { .. } => Some("pon"),
                Event::Agari { .. } => Some("agari"),
                _ => None,
            })
            .collect();

        assert!(event_types.contains(&"draw"));
        assert!(event_types.contains(&"discard"));
        assert!(event_types.contains(&"dora"));
        assert!(event_types.contains(&"reach"));
        assert!(event_types.contains(&"pon"));
        assert!(event_types.contains(&"agari"));
    }

    #[test]
    fn test_parse_error_cases() {
        // Test invalid seed format
        let bad_seed = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <INIT seed="0,0" ten="250,250,250,250" oya="0" hai0="0" hai1="1" hai2="2" hai3="3"/>
</mjloggm>"#;

        let cursor = Cursor::new(bad_seed.as_bytes());
        let result = parse_mjlog(cursor);
        assert!(result.is_err());

        // Test invalid ten format
        let bad_ten = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <INIT seed="0,0,0,1,2,52" ten="250,250" oya="0" hai0="0" hai1="1" hai2="2" hai3="3"/>
</mjloggm>"#;

        let cursor = Cursor::new(bad_ten.as_bytes());
        let result = parse_mjlog(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_ryuukyoku_types() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <GO type="169" lobby="0"/>
    <UN n0="Player1" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,M,M,M"/>
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0" hai1="1" hai2="2" hai3="3"/>
    <RYUUKYOKU ba="0,0" sc="250,0,250,0,250,0,250,0" type="yao9"/>
    <INIT seed="1,0,0,1,2,52" ten="250,250,250,250" oya="1" hai0="0" hai1="1" hai2="2" hai3="3"/>
    <RYUUKYOKU ba="0,0" sc="250,0,250,0,250,0,250,0" type="kaze4"/>
    <INIT seed="2,0,0,1,2,52" ten="250,250,250,250" oya="2" hai0="0" hai1="1" hai2="2" hai3="3"/>
    <RYUUKYOKU ba="0,0" sc="250,0,250,0,250,0,250,0" type="reach4"/>
    <INIT seed="3,0,0,1,2,52" ten="250,250,250,250" oya="3" hai0="0" hai1="1" hai2="2" hai3="3"/>
    <RYUUKYOKU ba="0,0" sc="250,0,250,0,250,0,250,0" type="ron3"/>
    <INIT seed="4,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0" hai1="1" hai2="2" hai3="3"/>
    <RYUUKYOKU ba="0,0" sc="250,0,250,0,250,0,250,0" type="kan4"/>
    <INIT seed="5,0,0,1,2,52" ten="250,250,250,250" oya="1" hai0="0" hai1="1" hai2="2" hai3="3"/>
    <RYUUKYOKU ba="0,0" sc="250,0,250,0,250,0,250,0" type="unknown"/>
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let result = parse_mjlog(cursor);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.rounds.len(), 6);

        // Check ryuukyoku types
        for (i, round) in output.rounds.iter().enumerate() {
            if let Some(Event::Ryuukyoku { reason, .. }) = round.events.last() {
                match i {
                    0 => assert!(matches!(reason, RyuukyokuReason::Yao9)),
                    1 => assert!(matches!(reason, RyuukyokuReason::Kaze4)),
                    2 => assert!(matches!(reason, RyuukyokuReason::Reach4)),
                    3 => assert!(matches!(reason, RyuukyokuReason::Ron3)),
                    4 => assert!(matches!(reason, RyuukyokuReason::Kan4)),
                    5 => assert!(matches!(reason, RyuukyokuReason::Normal)), // Unknown type defaults to Normal
                    _ => panic!("Unexpected round"),
                }
            }
        }
    }

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("%E3%83%86%E3%82%B9%E3%83%88"), "テスト");
        assert_eq!(percent_decode("Player1"), "Player1");
        assert_eq!(percent_decode(""), "");
    }

    #[test]
    fn test_parse_file_gz() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <GO type="169" lobby="0"/>
    <UN n0="Player1" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,F,M,F"/>
    <TAIKYOKU oya="0"/>
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0,4,8,12,16,20,24,28,32,36,40,44,48" hai1="1,5,9,13,17,21,25,29,33,37,41,45,49" hai2="2,6,10,14,18,22,26,30,34,38,42,46,50" hai3="3,7,11,15,19,23,27,31,35,39,43,47,51"/>
</mjloggm>"#;

        // Create a gzipped temporary file
        let mut input_file = NamedTempFile::with_suffix(".mjlog.gz").unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(mjlog_content.as_bytes()).unwrap();
        let compressed_data = encoder.finish().unwrap();
        input_file.write_all(&compressed_data).unwrap();
        input_file.flush().unwrap();

        let output_file = NamedTempFile::with_suffix(".json").unwrap();
        let options = ParserOptions {
            verbose: false,
            validate_schema: None,
        };

        // This should test the gz branch in parse_file
        let result = parse_file(input_file.path(), output_file.path(), &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_file_regular() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <GO type="169" lobby="0"/>
    <UN n0="Player1" n1="Player2" n2="Player3" n3="Player4" dan="1,2,3,4" rate="1500,1600,1700,1800" sx="M,F,M,F"/>
    <TAIKYOKU oya="0"/>
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0,4,8,12,16,20,24,28,32,36,40,44,48" hai1="1,5,9,13,17,21,25,29,33,37,41,45,49" hai2="2,6,10,14,18,22,26,30,34,38,42,46,50" hai3="3,7,11,15,19,23,27,31,35,39,43,47,51"/>
</mjloggm>"#;

        // Create a regular temporary file
        let mut input_file = NamedTempFile::with_suffix(".mjlog").unwrap();
        input_file.write_all(mjlog_content.as_bytes()).unwrap();
        input_file.flush().unwrap();

        let output_file = NamedTempFile::with_suffix(".json").unwrap();
        let options = ParserOptions {
            verbose: false,
            validate_schema: None,
        };

        // This should test the non-gz branch in parse_file
        let result = parse_file(input_file.path(), output_file.path(), &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_encoding_warnings() {
        // Create content with invalid Shift_JIS bytes to trigger encoding warnings
        let mut invalid_content =
            b"<?xml version=\"1.0\" encoding=\"Shift_JIS\"?>\n<mjloggm ver=\"2.3\">\n".to_vec();
        // Add some invalid bytes that will cause encoding warnings
        invalid_content.extend_from_slice(&[0xFF, 0xFE, 0xFD]); // Invalid Shift_JIS sequence
        invalid_content.extend_from_slice(b"\n</mjloggm>");

        let cursor = Cursor::new(invalid_content);
        let result = parse_mjlog(cursor);
        // Should still succeed but with encoding warnings
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_tags() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <UNKNOWN_TAG attr="value"/>
    <ANOTHER_UNKNOWN>content</ANOTHER_UNKNOWN>
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0,4,8,12,16,20,24,28,32,36,40,44,48" hai1="1,5,9,13,17,21,25,29,33,37,41,45,49" hai2="2,6,10,14,18,22,26,30,34,38,42,46,50" hai3="3,7,11,15,19,23,27,31,35,39,43,47,51"/>
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let result = parse_mjlog(cursor);
        assert!(result.is_ok());
    }

    #[test]
    fn test_draw_discard_edge_cases() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0,4,8,12,16,20,24,28,32,36,40,44,48" hai1="1,5,9,13,17,21,25,29,33,37,41,45,49" hai2="2,6,10,14,18,22,26,30,34,38,42,46,50" hai3="3,7,11,15,19,23,27,31,35,39,43,47,51"/>
    <T test="invalid"/>
    <U/>
    <V60 extra="attr"/>
    <W/>
    <D test="invalid"/>
    <E/>
    <F60 extra="attr"/>
    <G/>
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let result = parse_mjlog(cursor);
        assert!(result.is_ok());

        // Verify some events were captured despite edge cases
        let output = result.unwrap();
        assert!(!output.rounds.is_empty());
        let round = &output.rounds[0];
        // Should have some draw and discard events
        assert!(!round.events.is_empty());
    }

    #[test]
    fn test_parse_stream_error() {
        // Test invalid JSON serialization by using a mock writer that always fails
        struct FailingWriter;
        impl Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Write failed",
                ))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Flush failed",
                ))
            }
        }

        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="0" hai1="1" hai2="2" hai3="3"/>
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let failing_writer = FailingWriter;
        let options = ParserOptions {
            verbose: false,
            validate_schema: None,
        };

        let result = parse_stream(cursor, failing_writer, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_xml() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let result = parse_mjlog(cursor);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.mjlog_version, "2.3");
        assert!(output.rounds.is_empty());
    }

    #[test]
    fn test_invalid_tile_parsing() {
        let mjlog_content = r#"<?xml version="1.0" encoding="Shift_JIS"?>
<mjloggm ver="2.3">
    <INIT seed="0,0,0,1,2,52" ten="250,250,250,250" oya="0" hai0="invalid,abc,def" hai1="1" hai2="2" hai3="3"/>
</mjloggm>"#;

        let cursor = Cursor::new(mjlog_content.as_bytes());
        let result = parse_mjlog(cursor);
        // Should fail due to invalid tile parsing
        assert!(result.is_err());
    }
}
