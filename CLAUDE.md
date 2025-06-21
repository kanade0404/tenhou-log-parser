# CLAUDE.md - 天鳳mjlogパーサー開発ガイド

## プロジェクト概要
天鳳の対戦ログ（mjlog）を解析し、JSON形式に変換するRustライブラリ/CLIツールです。

## 実行コマンド
```bash
# ビルド
cargo build --release

# テスト実行
cargo test

# Lint & Format
cargo clippy -- -D warnings
cargo fmt -- --check

# CLI実行例
./target/release/mjlog-parser input.xml.gz -o output.json
```

## 重要なファイル
- `docs/`: 仕様書・設計書
- `src/models.rs`: JSON出力データ構造
- `src/parser.rs`: XMLパーサーコア
- `src/tile.rs`: 牌番号⇔文字列変換
- `tests/`: テストファイル
- `task.md`: 詳細タスクリスト

## 設計原則
1. **エラー処理**: thiserror + anyhowでエラー型を明確化
2. **所有権**: 借用を活用してメモリ効率を最適化
3. **テスト**: ユニット・統合・E2Eテストで品質担保
4. **ドキュメント**: コードコメントとREADMEで使用方法を明示

## 開発フロー
1. task.mdの各Phaseを順番に実行
2. 各機能実装後、必ずテストを作成・実行
3. Clippy・rustfmtで品質チェック
4. 実際の天鳳ログでE2Eテスト実行

## E2Eテストについて
- 実際の天鳳mjlogファイルをダウンロード
- 期待するJSON出力と比較検証
- tests/data/に実データ配置してテスト自動化

## 次にやるべきこと
現在Phase 2の基盤整備中。task.mdの未完了タスクを順次実行してください。