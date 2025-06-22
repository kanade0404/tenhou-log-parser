# プロジェクトコンテキスト: 天鳳 mjlog パーサー (Rust)

このドキュメントは、天鳳の対局ログ（mjlog）を解析し、JSON に変換する Rust 製パーサー開発プロジェクトの背景と目的、範囲をまとめたものです。

---

## 1. プロジェクト概要

* **プロジェクト名**: 天鳳 mjlog パーサー
* **言語**: Rust
* **目的**: 天鳳が出力する公式対局ログ（XML/GZIP 形式）の mjlog を読み込み、構造化された JSON データに変換するライブラリ／CLI ツールを提供。
* **利用シーン**:

  * 対局データの可視化・分析ダッシュボードへの取り込み
  * 機械学習パイプラインの前処理
  * 自動集計・統計ツール

---

## 2. ドメイン・入力仕様

* **mjlog ファイル**:

  * 拡張子 `.xml.gz` の GZIP 圧縮 XML
  * 文字コード: Shift\_JIS
  * ルート要素 `<mjloggm>` の属性:

    * `ver`, `shuffle`, `seed`, `ref`
  * 主な子要素: `<GO>`, `<UN>`, `<TAIKYOKU>`, `<INIT>`, `<T/U/V/W>`, `<D/E/F/G>`, `<N>`, `<DORA>`, `<REACH>`, `<AGARI>`, `<RYUUKYOKU>`
  * 牌番号 (0–135) → 牌文字列 (1m–9m,1p–9p,1s–9s,字牌)

---

## 3. 出力仕様

* **JSON フォーマット** (camelCase):

  * `mjlogVersion`, `gameId`, `rules`, `players`, `rounds`
  * 各 `rounds[i]` に `roundId`, `dealerSeat`, `init`, `events`
  * 各イベントを `type` フィールドで共通化 (`draw`, `discard`, `chi`, `pon`, `kan`, `dora`, `reach`, `agari`, `ryuukyoku`)

---

## 4. 開発体制・ステークホルダー

* **開発リーダー**: Seiya Sakata
* **実装担当**: Claude Code + レビュアー（GitHub PR ベース）
* **ユーザー**:

  * データ分析チーム
  * フロントエンド開発チーム（可視化）
  * 研究開発部門（機械学習用前処理）

---

## 5. 技術スタック・ライブラリ

* **言語**: Rust (stable)
* **主な依存**:

  * `serde` / `serde_json`: シリアライズ
  * `quick-xml`: XML パース
  * `anyhow`: エラー管理
  * `log` + `env_logger`: ロギング
* **開発補助**:

  * `rustfmt`, `clippy`（Lint & Format）
  * GitHub Actions CI
  * VSCode + rust-analyzer

---

## 6. アーキテクチャ概要

1. **Reader**: GZIP 解凍 → Shift\_JIS → `quick-xml::Reader`
2. **Tokenizer / Parser**: イベントタグごとにデシリアライズ
3. **ドメインモデル**: Rust 構造体 (`Game`, `Player`, `Round`, `Event` など)
4. **Post-processor**: 順序付け・集約・スコア計算補完
5. **Serializer**: `serde_json` で JSON 出力
6. **CLI**: `mjlog-parser input.xml.gz --output out.json`

---

## 7. 品質保証

* **Format/Lint**: 事前に設定した `rustfmt.toml` / `clippy.toml` による自動チェック
* **テスト**: サンプル mjlog を用いたユニットテスト／統合テスト
* **CI**: GitHub Actions でビルド・テスト・Lint・Format チェック

---

## 8. 今後の拡張予定

* JSON Schema の発行
* WebAssembly ビルド対応（ブラウザ連携）
* Kafka へのストリーミング出力モード
* GUI アプリケーション連携サンプル

以上のコンテキストをもとに、実装フェーズへ進んでください。

