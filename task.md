# 天鳳mjlogパーサー 詳細タスクリスト

## Phase 1: プロジェクト基盤構築 ✅
- [x] Cargo.toml作成（依存関係設定）
- [x] rust-toolchain.toml作成（toolchain固定）
- [x] rustfmt.toml作成（フォーマット設定）
- [x] clippy.toml作成（lint設定）
- [x] src/ディレクトリ作成
- [x] 基本計画書(plan.md)作成

## Phase 2: モジュール設計・エラー処理 ✅

### 2.1 プロジェクト構造整備
- [x] src/lib.rs作成（ライブラリルート、pub use設定）
- [x] src/main.rs作成（CLIエントリーポイント）
- [x] tests/ディレクトリ作成
- [x] tests/data/ディレクトリ作成（テストデータ用）
- [x] tests/helpers.rs作成（テストユーティリティ）

### 2.2 エラー処理設計
- [x] src/error.rs作成
  - [x] ParserError列挙型定義（thiserror使用）
  - [x] IoError, XmlError, SchemaError, EncodingError variants
  - [x] From<std::io::Error>等の変換実装
  - [x] Display, Debug実装

### 2.3 牌変換ユーティリティ
- [x] src/tile.rs作成
  - [x] tile_id_to_string関数実装（0-135 → 牌文字列）
  - [x] tile_string_to_id関数実装（逆変換）
  - [x] TileType列挙型定義
  - [x] ユニットテスト追加（#[cfg(test)]）

## Phase 3: データモデル定義 ✅

### 3.1 基本データ構造
- [x] src/models.rs改善
  - [x] ParserOutput構造体（mjlogVersion, gameId, rules, players, rounds）
  - [x] Rules構造体（typeFlags, lobbyId）
  - [x] Player構造体（seat, playerId, rank, rate, gender）
  - [x] Round構造体（roundId, dealerSeat, init, events）
  - [x] Init構造体（全フィールド）

### 3.2 イベントモデル
- [x] Event列挙型の詳細実装
  - [x] DrawEvent（seat, tile）
  - [x] DiscardEvent（seat, tile, isRiichi）
  - [x] ChiEvent（who, tiles[3], from）
  - [x] PonEvent（who, tiles[3], from）
  - [x] KanEvent（who, tiles, kanType, from?）
  - [x] DoraEvent（indicator）
  - [x] ReachEvent（who, step, scores[4]）
  - [x] AgariEvent（who, from, han, fu, yakus, doraCount, scores[4]）
  - [x] RyuukyokuEvent（reason, scores[4]）

### 3.3 補助構造体
- [x] Yaku構造体（name, value）
- [x] KanType列挙型（Ankan, Minkan, Kakan）
- [x] RyuukyokuReason列挙型

## Phase 4: XMLパーサー実装 ✅

### 4.1 入力処理
- [x] src/parser.rs内に統合
  - [x] ファイル読み込み関数（.xml/.xml.gz対応）
  - [x] GZIP解凍処理（flate2使用）
  - [x] Shift_JIS→UTF-8変換（encoding_rs使用）
  - [x] BufReader最適化

### 4.2 XMLパーサーコア
- [x] src/parser.rs作成
  - [x] MjlogParser構造体定義
  - [x] parse_mjlog関数（メイン処理）
  - [x] quick-xml::Reader設定（バッファサイズ調整）

### 4.3 各XMLタグのパース処理
- [x] parse_mjloggm_attrs関数（ver, shuffle, seed, ref）
- [x] parse_go_tag関数（type, lobby）
- [x] parse_un_tag関数（n0-n3, dan, rate, sx）
- [x] parse_taikyoku_tag関数（oya）
- [x] parse_init_tag関数（seed, ten, hai0-hai3）
- [x] parse_draw_tag関数（T/U/V/W）
- [x] parse_discard_tag関数（D/E/F/G）
- [x] parse_naki_tag関数（N）
- [x] parse_dora_tag関数（DORA）
- [x] parse_reach_tag関数（REACH）
- [x] parse_agari_tag関数（AGARI）
- [x] parse_ryuukyoku_tag関数（RYUUKYOKU）

### 4.4 補助パース機能
- [x] 牌番号配列パース（hai0-hai3, machi等）
- [x] スコア配列パース（ten, sc）
- [x] ビットフラグパース（type, yaku）
- [x] seed文字列パース（局順,本場,供託,サイコロ×2,ドラ）

## Phase 5: JSON出力・シリアライゼーション ✅

### 5.1 出力処理
- [x] src/parser.rs内に統合
  - [x] JSON出力関数（serde_json::to_writer_pretty）
  - [x] ファイル書き込み処理
  - [x] --force上書きチェック
  - [x] --stream標準出力対応

### 5.2 スキーマ検証（オプション）
- [ ] JSON Schema定義ファイル作成
- [ ] jsonschemaクレートによる検証機能
- [ ] --schemaオプション対応

## Phase 6: CLI実装 ✅

### 6.1 CLI引数処理
- [x] src/main.rs内に統合
  - [x] Clap Args構造体定義
  - [x] input, output, force, verbose, stream, schema オプション
  - [x] 引数検証ロジック
  - [x] ヘルプメッセージ設定

### 6.2 メイン処理
- [x] src/main.rs実装
  - [x] CLI引数パース
  - [x] ログ設定（env_logger）
  - [x] パーサー呼び出し
  - [x] エラーハンドリング（anyhow::Result）

### 6.3 ライブラリAPI
- [x] src/lib.rs実装
  - [x] pub use でAPI公開
  - [x] parse_stream関数
  - [x] parse_file関数
  - [x] ParserOptions構造体

## Phase 7: テスト実装 ✅

### 7.1 ユニットテスト
- [x] src/tile.rs - テスト（#[cfg(test)]）
  - [x] tile_id_to_string正常系テスト
  - [x] tile_id_to_string境界値テスト
  - [x] tile_string_to_id正常系テスト
- [x] src/models.rs - テスト
  - [x] serde serialize/deserializeテスト
  - [x] Event列挙型テスト
- [x] src/parser.rs - テスト
  - [x] 各パース関数の単体テスト
  - [x] XMLタグ別テストケース
  - [x] エラー処理テスト

### 7.2 統合テスト
- [x] tests/e2e_test.rs作成
  - [x] CLI実行テスト（Command実行テスト）
  - [x] サンプルmjlogでの完全パーステスト
  - [x] JSON出力内容検証
  - [x] ファイル出力モードテスト
  - [x] verboseモードテスト
  - [x] エラーハンドリングテスト
- [x] テストデータ準備
  - [x] tests/data/sample.xml作成
  - [x] tests/data/sample_complex.xml作成
  - [x] tests/data/sample_complex.mjlog作成（GZIP版）

### 7.3 ドキュメントテスト
- [ ] 各public関数に/// docコメント追加
- [ ] コードブロック例の追加
- [ ] cargo test doctest実行確認

### 7.4 プロパティテスト（オプション）
- [ ] proptest依存関係追加
- [ ] ランダムmjlog生成テスト
- [ ] parse→serialize→parse一貫性テスト

## Phase 8: 品質保証・CI

### 8.1 コード品質チェック
- [ ] cargo clippy -- -D warnings 通過確認
- [ ] cargo fmt -- --check 通過確認
- [ ] cargo test 全パス確認
- [ ] cargo doc 正常生成確認

### 8.2 カバレッジ測定
- [ ] cargo-tarpaulin導入
- [ ] カバレッジレポート生成
- [ ] 80%以上のカバレッジ達成

### 8.3 CI/CD設定
- [ ] .github/workflows/ci.yml作成
  - [ ] multiple OS対応（ubuntu, macos, windows）
  - [ ] Rust stable/beta/nightly テスト
  - [ ] lint, format, test, build各ステップ
  - [ ] カバレッジレポートPR投稿

### 8.4 パフォーマンステスト
- [ ] 大きなmjlogファイルでのベンチマーク
- [ ] cargo flamegraphによるプロファイリング
- [ ] メモリ使用量測定

## Phase 9: ドキュメント・配布

### 9.1 ドキュメント整備
- [ ] README.md作成
  - [ ] インストール方法
  - [ ] 使用方法（CLI/Library）
  - [ ] サンプルコード
  - [ ] パフォーマンス仕様
- [ ] CHANGELOG.md作成
- [ ] LICENSE ファイル追加

### 9.2 配布準備
- [ ] cargo publish準備
- [ ] crates.io メタデータ設定
- [ ] GitHub Release設定

## Phase 10: 追加機能（将来対応）

### 10.1 拡張機能
- [ ] WebAssembly対応
- [ ] 非同期I/O対応（tokio）
- [ ] Kafka出力対応
- [ ] 複数ファイル一括処理

### 10.2 GUI/Web対応
- [ ] Web API サーバー実装
- [ ] フロントエンド連携サンプル

---

## チェックポイント

各Phase完了時に以下を確認：
- [ ] Clippy警告ゼロ
- [ ] テスト全パス
- [ ] ドキュメント更新
- [ ] git commit & push

緊急度: **High** = 基本機能, **Medium** = 品質向上, **Low** = 将来拡張