use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserOutput {
    #[serde(rename = "mjlogVersion")]
    pub mjlog_version: String,
    #[serde(rename = "gameId")]
    pub game_id: String,
    pub rules: Rules,
    pub players: Vec<Player>,
    pub rounds: Vec<Round>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    #[serde(rename = "typeFlags")]
    pub type_flags: u32,
    #[serde(rename = "lobbyId")]
    pub lobby_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub seat: u8,
    #[serde(rename = "playerId")]
    pub player_id: String,
    pub rank: u32,
    pub rate: u32,
    pub gender: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    #[serde(rename = "roundId")]
    pub round_id: String,
    #[serde(rename = "dealerSeat")]
    pub dealer_seat: u8,
    pub init: Init,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    #[serde(rename = "roundNumber")]
    pub round_number: u32,
    pub honba: u32,
    pub kyoutaku: u32,
    pub dice: [u32; 2],
    #[serde(rename = "doraIndicator")]
    pub dora_indicator: u32,
    #[serde(rename = "initialScores")]
    pub initial_scores: [i32; 4],
    #[serde(rename = "initialHands")]
    pub initial_hands: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "draw")]
    Draw { seat: u8, tile: String },
    #[serde(rename = "discard")]
    Discard {
        seat: u8,
        tile: String,
        #[serde(rename = "isRiichi")]
        is_riichi: bool,
    },
    #[serde(rename = "chi")]
    Chi {
        who: u8,
        tiles: [String; 3],
        from: u8,
    },
    #[serde(rename = "pon")]
    Pon {
        who: u8,
        tiles: [String; 3],
        from: u8,
    },
    #[serde(rename = "kan")]
    Kan {
        who: u8,
        tiles: Vec<String>,
        #[serde(rename = "kanType")]
        kan_type: KanType,
        from: Option<u8>,
    },
    #[serde(rename = "dora")]
    Dora { indicator: String },
    #[serde(rename = "reach")]
    Reach {
        who: u8,
        step: u8,
        scores: [i32; 4],
    },
    #[serde(rename = "agari")]
    Agari {
        who: u8,
        from: u8,
        han: u32,
        fu: u32,
        yakus: Vec<Yaku>,
        #[serde(rename = "doraCount")]
        dora_count: u32,
        scores: [i32; 4],
    },
    #[serde(rename = "ryuukyoku")]
    Ryuukyoku {
        reason: RyuukyokuReason,
        scores: [i32; 4],
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Yaku {
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KanType {
    Ankan,  // 暗槓
    Minkan, // 明槓
    Kakan,  // 加槓
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RyuukyokuReason {
    #[serde(rename = "nm")]
    Normal,       // 通常の流局
    #[serde(rename = "yao9")]
    Yao9,         // 九種九牌
    #[serde(rename = "kaze4")]
    Kaze4,        // 四風連打
    #[serde(rename = "reach4")]
    Reach4,       // 四人リーチ
    #[serde(rename = "ron3")]
    Ron3,         // 三人和了
    #[serde(rename = "kan4")]
    Kan4,         // 四槓散了
}

