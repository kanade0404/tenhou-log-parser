天鳳ログ（mjlog）仕様およびJSON出力データ構造仕様

本ドキュメントでは、天鳳が生成する対局ログ（mjlog）の入力仕様と、パーサーが最終的に出力する JSON データ構造の仕様をまとめます。

⸻

1. 入力仕様：天鳳 mjlog（XML）

1.1 基本情報
	•	ファイル形式: GZIP 圧縮された XML (.xml.gz) または非圧縮 XML (.xml)
	•	文字コード: Shift_JIS → UTF-8 へ変換
	•	ルート要素: <mjloggm>
	•	属性:
	•	ver (フォーマットバージョン)
	•	shuffle (シャッフル乱数表現)
	•	seed (シード情報: 局順・本場・供託・サイコロ)
	•	ref (未使用)

1.2 主要タグと属性

タグ名	主な属性・説明
<GO>	type (ルールビットフラグ), lobby (大会ID)
<UN>	n0–n3 (各席の天鳳ID), dan (段位0–3), rate (レート0–3), sx (性別M/F)
<TAIKYOKU>	oya (起家席番号：0–3)
<INIT>	seed (局順,本場,供託,サイコロ目×2,ドラ表示牌番号), ten (初期持ち点×4), oya, hai0–hai3 (配牌13枚)
<T/U/V/W>	プレイヤー0–3 のツモ牌 (T: 0, U:1, V:2, W:3) 各牌番号0–135
<D/E/F/G>	プレイヤー0–3 の打牌 (D:0, E:1, F:2, G:3)
<N>	副露タグ: who (実行者席番号), meld (面子牌番号列)
<DORA>	新ドラ表示: hai (牌番号)
<REACH>	立直: who, step (1=宣言, 2=成立), ten (宣言後持ち点×4)
<AGARI>	和了: ba (積棒,リーチ棒), hai (手牌列), machi (待ち牌番号), ten (符,和了点,満貫区分), yaku, doraHai/doraHaiUra, who, fromWho, sc (点数変動)
<RYUUKYOKU>	流局: ba, sc (収支), type (nm/yao9/kaze4/reach4/ron3/kan4)

1.3 牌番号→牌文字列マッピング

let id in 0..135:
  tileType = id / 4  (整数除算 0–33)
  0–8   → 1m–9m
  9–17  → 1p–9p
  18–26 → 1s–9s
  27    → east
  28    → south
  29    → west
  30    → north
  31    → white
  32    → green
  33    → red

同一牌は4枚存在し、番号は4枚ずつ連続。

⸻

2. 出力仕様：JSON データ構造

パーサーは以下の構造で JSON を生成します。TypeScript 型定義例を併記。

2.1 トップレベル

interface ParserOutput {
  mjlogVersion: string;      // <mjloggm ver>
  gameId: string;            // 任意生成の一意ID
  rules: Rules;
  players: Player[];
  rounds: Round[];
}

2.2 ルール情報

interface Rules {
  typeFlags: number;         // <GO type>
  lobbyId: number | null;    // <GO lobby>
}

2.3 プレイヤー情報

interface Player {
  seat: 0 | 1 | 2 | 3;
  playerId: string;          // 天鳳ID
  rank: number;              // 段位
  rate: number;              // レート
  gender: "M" | "F";
}

2.4 局情報

interface Round {
  roundId: string;           // "東1局" など
  dealerSeat: 0 | 1 | 2 | 3; // 起家席番号
  init: Init;
  events: Event[];           // 順序通り
}

interface Init {
  roundNumber: number;       // 局順
  honba: number;             // 本場数
  kyoutaku: number;          // 供託棒
  dice: [number, number];    // サイコロ目
  doraIndicator: number;     // ドラ表示牌番号
  initialScores: [number, number, number, number];
  initialHands: string[][];  // 各席の配牌文字列配列
}

2.5 イベント情報

共通フィールド type で識別。

type Event =
  | DrawEvent
  | DiscardEvent
  | ChiEvent
  | PonEvent
  | KanEvent
  | DoraEvent
  | ReachEvent
  | AgariEvent
  | RyuukyokuEvent;

各イベント型例：

interface DrawEvent { type: "draw"; seat: 0|1|2|3; tile: string; }
interface DiscardEvent { type: "discard"; seat: 0|1|2|3; tile: string; isRiichi: boolean; }
interface ChiEvent { type: "chi"; who: 0|1|2|3; tiles: [string,string,string]; from: 0|1|2|3; }
interface PonEvent { type: "pon"; who: 0|1|2|3; tiles: [string,string,string]; from: 0|1|2|3; }
interface KanEvent { type: "kan"; who: 0|1|2|3; tiles: string[]; kanType: "ankan"|"minkan"|"kakan"; from?: 0|1|2|3; }
interface DoraEvent { type: "dora"; indicator: string; }
interface ReachEvent { type: "reach"; who: 0|1|2|3; step: 1|2; scores: [number,number,number,number]; }
interface AgariEvent {
  type: "agari"; who: 0|1|2|3; from: 0|1|2|3;
  han: number; fu: number;
  yakus: { name: string; value: number; }[];
  doraCount: number; scores: [number,number,number,number];
}
interface RyuukyokuEvent { type: "ryuukyoku"; reason: string; scores: [number,number,number,number]; }


⸻

以上の仕様に沿って実装すれば、天鳳 mjlog の構造を忠実に JSON で表現できます。必要に応じて JSON Schema や TypeScript の type／interface をプロジェクトに追加してください。
