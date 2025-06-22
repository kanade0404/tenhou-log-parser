use std::borrow::Cow;

use crate::error::{ParserError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Man(u8), // 1m-9m
    Pin(u8), // 1p-9p
    Sou(u8), // 1s-9s
    East,    // 東
    South,   // 南
    West,    // 西
    North,   // 北
    White,   // 白
    Green,   // 発
    Red,     // 中
}

/// Convert tile ID (0-135) to tile string representation
///
/// # Examples
/// ```
/// use tenhou_log_parser::tile_id_to_string;
/// assert_eq!(tile_id_to_string(0), "1m");
/// assert_eq!(tile_id_to_string(31 * 4), "white");
/// ```
pub fn tile_id_to_string(id: u32) -> Cow<'static, str> {
    let tile_type = id / 4;
    match tile_type {
        0..=8 => Cow::Owned(format!("{}m", tile_type + 1)),
        9..=17 => Cow::Owned(format!("{}p", tile_type - 8)),
        18..=26 => Cow::Owned(format!("{}s", tile_type - 17)),
        27 => Cow::Borrowed("east"),
        28 => Cow::Borrowed("south"),
        29 => Cow::Borrowed("west"),
        30 => Cow::Borrowed("north"),
        31 => Cow::Borrowed("white"),
        32 => Cow::Borrowed("green"),
        33 => Cow::Borrowed("red"),
        _ => Cow::Owned(format!("unknown_{}", id)),
    }
}

/// Convert tile string to tile ID (0-135)
/// Returns the first ID for the tile type (multiple copies exist)
///
/// # Examples
/// ```
/// use tenhou_log_parser::tile_string_to_id;
/// assert_eq!(tile_string_to_id("1m").unwrap(), 0);
/// assert_eq!(tile_string_to_id("white").unwrap(), 124);
/// ```
pub fn tile_string_to_id(tile: &str) -> Result<u32> {
    let tile_type =
        match tile {
            s if s.ends_with('m') && s.len() == 2 => {
                let num = s.chars().next().unwrap().to_digit(10).ok_or_else(|| {
                    ParserError::invalid_format(format!("Invalid man tile: {}", s))
                })?;
                if (1..=9).contains(&num) {
                    num - 1
                } else {
                    return Err(ParserError::invalid_format(format!(
                        "Invalid man tile: {}",
                        s
                    )));
                }
            }
            s if s.ends_with('p') && s.len() == 2 => {
                let num = s.chars().next().unwrap().to_digit(10).ok_or_else(|| {
                    ParserError::invalid_format(format!("Invalid pin tile: {}", s))
                })?;
                if (1..=9).contains(&num) {
                    num - 1 + 9
                } else {
                    return Err(ParserError::invalid_format(format!(
                        "Invalid pin tile: {}",
                        s
                    )));
                }
            }
            s if s.ends_with('s') && s.len() == 2 => {
                let num = s.chars().next().unwrap().to_digit(10).ok_or_else(|| {
                    ParserError::invalid_format(format!("Invalid sou tile: {}", s))
                })?;
                if (1..=9).contains(&num) {
                    num - 1 + 18
                } else {
                    return Err(ParserError::invalid_format(format!(
                        "Invalid sou tile: {}",
                        s
                    )));
                }
            }
            "east" => 27,
            "south" => 28,
            "west" => 29,
            "north" => 30,
            "white" => 31,
            "green" => 32,
            "red" => 33,
            _ => {
                return Err(ParserError::invalid_format(format!(
                    "Unknown tile: {}",
                    tile
                )))
            }
        };

    Ok(tile_type * 4)
}

/// Convert tile type to TileType enum
pub fn tile_id_to_type(id: u32) -> Result<TileType> {
    let tile_type = id / 4;
    match tile_type {
        0..=8 => Ok(TileType::Man((tile_type + 1) as u8)),
        9..=17 => Ok(TileType::Pin((tile_type - 8) as u8)),
        18..=26 => Ok(TileType::Sou((tile_type - 17) as u8)),
        27 => Ok(TileType::East),
        28 => Ok(TileType::South),
        29 => Ok(TileType::West),
        30 => Ok(TileType::North),
        31 => Ok(TileType::White),
        32 => Ok(TileType::Green),
        33 => Ok(TileType::Red),
        _ => Err(ParserError::InvalidTileId(id)),
    }
}

/// Parse a comma-separated list of tile IDs to tile strings
pub fn parse_tile_list(tiles: &str) -> Result<Vec<String>> {
    if tiles.is_empty() {
        return Ok(vec![]);
    }

    tiles
        .split(',')
        .map(|s| {
            let id = s
                .parse::<u32>()
                .map_err(|_| ParserError::invalid_format(format!("Invalid tile ID: {}", s)))?;
            Ok(tile_id_to_string(id).into_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_id_to_string() {
        assert_eq!(tile_id_to_string(0), "1m");
        assert_eq!(tile_id_to_string(4), "2m");
        assert_eq!(tile_id_to_string(32), "9m");
        assert_eq!(tile_id_to_string(36), "1p");
        assert_eq!(tile_id_to_string(68), "9p");
        assert_eq!(tile_id_to_string(72), "1s");
        assert_eq!(tile_id_to_string(104), "9s");
        assert_eq!(tile_id_to_string(108), "east");
        assert_eq!(tile_id_to_string(112), "south");
        assert_eq!(tile_id_to_string(116), "west");
        assert_eq!(tile_id_to_string(120), "north");
        assert_eq!(tile_id_to_string(124), "white");
        assert_eq!(tile_id_to_string(128), "green");
        assert_eq!(tile_id_to_string(132), "red");

        // Test invalid tile IDs - should return "unknown_<id>"
        assert_eq!(tile_id_to_string(136), "unknown_136");
        assert_eq!(tile_id_to_string(1000), "unknown_1000");
        assert_eq!(tile_id_to_string(9999), "unknown_9999");
    }

    #[test]
    fn test_tile_string_to_id() {
        assert_eq!(tile_string_to_id("1m").unwrap(), 0);
        assert_eq!(tile_string_to_id("2m").unwrap(), 4);
        assert_eq!(tile_string_to_id("9m").unwrap(), 32);
        assert_eq!(tile_string_to_id("1p").unwrap(), 36);
        assert_eq!(tile_string_to_id("9p").unwrap(), 68);
        assert_eq!(tile_string_to_id("1s").unwrap(), 72);
        assert_eq!(tile_string_to_id("9s").unwrap(), 104);
        assert_eq!(tile_string_to_id("east").unwrap(), 108);
        assert_eq!(tile_string_to_id("south").unwrap(), 112);
        assert_eq!(tile_string_to_id("west").unwrap(), 116);
        assert_eq!(tile_string_to_id("north").unwrap(), 120);
        assert_eq!(tile_string_to_id("white").unwrap(), 124);
        assert_eq!(tile_string_to_id("green").unwrap(), 128);
        assert_eq!(tile_string_to_id("red").unwrap(), 132);
    }

    #[test]
    fn test_tile_string_to_id_invalid() {
        // Test invalid man tiles
        assert!(tile_string_to_id("0m").is_err());
        assert!(tile_string_to_id("10m").is_err());
        assert!(tile_string_to_id("am").is_err()); // Non-digit character

        // Test invalid pin tiles
        assert!(tile_string_to_id("0p").is_err());
        assert!(tile_string_to_id("10p").is_err());
        assert!(tile_string_to_id("ap").is_err()); // Non-digit character

        // Test invalid sou tiles
        assert!(tile_string_to_id("0s").is_err());
        assert!(tile_string_to_id("10s").is_err());
        assert!(tile_string_to_id("as").is_err()); // Non-digit character

        // Test completely invalid strings
        assert!(tile_string_to_id("invalid").is_err());
        assert!(tile_string_to_id("").is_err());
        assert!(tile_string_to_id("xyz").is_err());
    }

    #[test]
    fn test_parse_tile_list() {
        assert_eq!(parse_tile_list("").unwrap(), Vec::<String>::new());
        assert_eq!(parse_tile_list("0,4,8").unwrap(), vec!["1m", "2m", "3m"]);
        assert_eq!(
            parse_tile_list("108,112,116,120").unwrap(),
            vec!["east", "south", "west", "north"]
        );
    }

    #[test]
    fn test_parse_tile_list_invalid() {
        assert!(parse_tile_list("0,invalid,8").is_err());
        assert!(parse_tile_list("abc").is_err());
    }

    #[test]
    fn test_tile_id_to_type() {
        assert_eq!(tile_id_to_type(0).unwrap(), TileType::Man(1));
        assert_eq!(tile_id_to_type(32).unwrap(), TileType::Man(9));
        assert_eq!(tile_id_to_type(36).unwrap(), TileType::Pin(1));
        assert_eq!(tile_id_to_type(68).unwrap(), TileType::Pin(9));
        assert_eq!(tile_id_to_type(72).unwrap(), TileType::Sou(1));
        assert_eq!(tile_id_to_type(104).unwrap(), TileType::Sou(9));
        assert_eq!(tile_id_to_type(108).unwrap(), TileType::East);
        assert_eq!(tile_id_to_type(112).unwrap(), TileType::South);
        assert_eq!(tile_id_to_type(116).unwrap(), TileType::West);
        assert_eq!(tile_id_to_type(120).unwrap(), TileType::North);
        assert_eq!(tile_id_to_type(124).unwrap(), TileType::White);
        assert_eq!(tile_id_to_type(128).unwrap(), TileType::Green);
        assert_eq!(tile_id_to_type(132).unwrap(), TileType::Red);
    }

    #[test]
    fn test_tile_id_to_type_invalid() {
        assert!(tile_id_to_type(136).is_err());
        assert!(tile_id_to_type(1000).is_err());
    }
}
