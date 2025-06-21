# Parser IO 仕様書

Rust 製 mjlog パーサーの入出力（IO）に関する仕様を定義します。

---

## 1. CLI インターフェース

### コマンド概要

```bash
mjlog-parser [OPTIONS] <INPUT>
```

* `<INPUT>`: 解析対象の mjlog ファイル（`.xml`または`.xml.gz`）
* `OPTIONS`: 以下のオプションをサポート

| オプション           | 説明                              | デフォルト          |
| --------------- | ------------------------------- | -------------- |
| `-o, --output`  | 出力ファイルパス（JSON）                  | `<INPUT>.json` |
| `-f, --force`   | 既存の出力を上書き                       | false          |
| `-v, --verbose` | 詳細ログを有効化（INFO→DEBUG）            | false          |
| `--stream`      | 標準出力ストリームモード（ファイル出力せず stdout）   | false          |
| `--schema`      | JSON Schema ファイルパスを指定し、出力をバリデート | なし             |

### 実行例

```bash
# デフォルト出力
mjlog-parser game1.xml.gz
# 上書き許可＆標準出力
mjlog-parser -f --stream -v game1.xml
# Schema validation
mjlog-parser --schema schema.json game1.xml.gz
```

---

## 2. ライブラリ API

パーサーをライブラリとして組み込む場合の主要関数シグネチャを定義します。

```rust
/// 入力ストリームから JSON 文字列を生成する。
///
/// - `reader`: GzDecoder または標準リーダー。
/// - `writer`: JSON 文字列出力先。
/// - `options`: パーサー設定（verbose, schema path）
///
pub fn parse_stream<R: Read, W: Write>(
    reader: R,
    writer: W,
    options: &ParserOptions
) -> Result<(), ParserError>;

/// ファイルパス指定でパースを実行する。
///
/// - `input_path`: `.xml` or `.xml.gz`。
/// - `output_path`: 出力先 JSON ファイル。

pub fn parse_file(
    input_path: &Path,
    output_path: &Path,
    options: &ParserOptions
) -> Result<(), ParserError>;

/// パーサーオプション
#[derive(Debug, Clone)]
pub struct ParserOptions {
    pub verbose: bool,
    pub validate_schema: Option<PathBuf>,
}

/// エラー型
#[derive(thiserror::Error, Debug)]
pub enum ParserError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML Parse Error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Schema Validation Error: {0}")]
    Schema(String),
}
```

---

## 3. ファイル入出力処理

1. **入力読み込み**

   * 拡張子が `.gz` の場合は `flate2::read::GzDecoder` で解凍しつつ `Shift_JIS` → UTF-8 変換。
   * `.xml` の場合は直接 `Shift_JIS` → UTF-8 変換。
2. **出力書き込み**

   * `--stream` モード: `writer` に直接書き込む（主に CLI 標準出力）。
   * 通常モード: 一時ファイルに書き出し、終了時に `force` フラグ確認後に移動。
3. **エラー処理**

   * 入力ファイル未検出: `ParserError::Io`。
   * XML 構文エラー: `ParserError::Xml`。
   * スキーマ検証エラー: `ParserError::Schema`。
   * ログは `log` クレート経由で INFO/ERROR レベル出力。

---

## 4. JSON 出力フォーマット

前述の出力仕様に準拠。

* パース完了後に `serde_json::to_writer_pretty` を使用して整形。
* `schema` オプション指定時は、`jsonschema` クレートで検証。

---

## 5. パフォーマンスとストリーミング

* メモリ効率向上のため、`quick-xml::Reader` のイベント駆動パースを採用。
* イベントごとにシリアライズし、バッファリングされた `writer` にフラッシュ。

以上がパーサーの IO 仕様です。実装時にこの仕様に従ってコードを構築してください。

