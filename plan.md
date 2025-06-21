# 天鳳mjlogパーサー実装計画

## 1. プロジェクト概要
- **言語**: Rust
- **目的**: 天鳳の対局ログ（mjlog）を解析し、JSON形式に変換するライブラリ/CLIツール
- **入力**: `.xml.gz`または`.xml`（Shift_JIS、GZIP圧縮XML）
- **出力**: 構造化されたJSON（camelCase）

## 2. アーキテクチャ設計

### 2.1 モジュール構成
```
src/
├── main.rs           # CLI エントリーポイント
├── lib.rs            # ライブラリのルート
├── models.rs         # データ構造定義（JSON出力用）
├── parser.rs         # XMLパーサー（メインロジック）
├── tile.rs           # 牌番号⇔牌文字列変換
├── error.rs          # エラー型定義
└── cli.rs           # CLI引数処理
```

### 2.2 データフロー
1. **入力処理**: ファイル読み込み → GZIP解凍 → Shift_JIS→UTF-8変換
2. **XMLパース**: quick-xmlでイベント駆動パース
3. **データ変換**: XML要素 → Rustモデル構造体
4. **JSON出力**: serdeでシリアライズ → ファイル書き込み

## 3. 実装フェーズ

### Phase 1: 基盤構築 ✅
- [x] プロジェクト初期化（Cargo.toml、rust-toolchain.toml）
- [x] Lint/Format設定（rustfmt.toml、clippy.toml）
- [x] ディレクトリ構造作成

### Phase 2: データモデル定義 🔄
- [x] JSON出力用構造体定義（models.rs）
- [ ] エラー型定義（error.rs）
- [ ] 牌変換ユーティリティ（tile.rs）

### Phase 3: コアパーサー実装
- [ ] XMLイベントパーサー（parser.rs）
- [ ] mjlog要素の解析ロジック
  - [ ] `<mjloggm>`属性パース
  - [ ] `<GO>`, `<UN>`, `<TAIKYOKU>`パース
  - [ ] `<INIT>`（局開始）パース
  - [ ] `<T/U/V/W>`（ツモ）パース
  - [ ] `<D/E/F/G>`（打牌）パース
  - [ ] `<N>`（副露）パース
  - [ ] `<DORA>`パース
  - [ ] `<REACH>`パース
  - [ ] `<AGARI>`パース
  - [ ] `<RYUUKYOKU>`パース

### Phase 4: CLI実装
- [ ] CLI引数処理（cli.rs）
- [ ] メイン関数（main.rs）
- [ ] ライブラリインターフェース（lib.rs）

### Phase 5: テスト・品質保証
- [ ] ユニットテスト
- [ ] 統合テスト（サンプルmjlogファイル）
- [ ] エラーハンドリングテスト
- [ ] CI設定（GitHub Actions）

## 4. 技術的考慮事項

### 4.1 文字エンコーディング
- Shift_JIS → UTF-8変換に`encoding_rs`クレートを使用
- GZIP解凍に`flate2`クレートを使用

### 4.2 XMLパース戦略
- `quick-xml`のイベント駆動パースを採用
- メモリ効率を重視し、ストリーミング処理
- 各XMLタグを対応するEventに変換

### 4.3 エラー処理
- `thiserror`でカスタムエラー型定義
- `anyhow`で上位レベルエラー処理
- ログ出力は`log`+`env_logger`

### 4.4 CLI設計
```bash
mjlog-parser [OPTIONS] <INPUT>

OPTIONS:
  -o, --output <FILE>     出力ファイルパス（デフォルト: <INPUT>.json）
  -f, --force            既存ファイル上書き許可
  -v, --verbose          詳細ログ
  --stream               標準出力モード
  --schema <FILE>        JSON Schema検証
```

## 5. 実装優先度

### High Priority
1. **基本パーサー機能**: mjlog → JSON変換の核心機能
2. **CLI基本機能**: ファイル入出力、基本オプション
3. **エラー処理**: 適切なエラーメッセージとハンドリング

### Medium Priority
1. **詳細ログ機能**: デバッグ用ログ出力
2. **テストスイート**: 主要機能のテストカバレッジ
3. **パフォーマンス最適化**: 大きなファイルの処理速度向上

### Low Priority
1. **JSON Schema検証**: 出力データの検証機能
2. **ストリームモード**: 標準出力への直接出力
3. **CI/CD設定**: 自動テスト・ビルド環境

## 6. 想定される課題と対策

### 6.1 文字エンコーディング問題
- **課題**: Shift_JISの正確な変換
- **対策**: `encoding_rs`での適切なエラーハンドリング

### 6.2 XMLフォーマットの多様性
- **課題**: mjlogの微妙なフォーマット差異
- **対策**: 寛容なパーサー設計、警告ログでの異常検出

### 6.3 メモリ使用量
- **課題**: 大きなログファイルでのメモリ不足
- **対策**: ストリーミング処理、イベント駆動パース

### 6.4 パフォーマンス
- **課題**: 大量ファイル処理の速度
- **対策**: 効率的なXMLパース、並列処理検討

## 7. 品質基準

### 7.1 コード品質
- Clippyでlint: `cargo clippy -- -D warnings`
- Rustfmtでformat: `cargo fmt -- --check`
- 単体テストカバレッジ80%以上

### 7.2 機能品質
- 正常系: 標準的なmjlogファイルの100%変換成功
- 異常系: 不正フォーマットでの適切なエラー出力
- パフォーマンス: 1MBのmjlogファイルを5秒以内で処理

この計画に基づいて段階的に実装を進めます。