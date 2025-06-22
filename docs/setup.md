# Rust 環境構築ガイド

このドキュメントでは、天鳳 mjlog パーサーを Rust で実装するための開発環境構築手順をまとめます。特に lint（Clippy）と format（rustfmt）の自動化／厳格化に重点を置いています。

---

## 1. 前提条件

* OS: Windows / macOS / Linux（任意）
* インターネット接続
* ターミナル操作に慣れていること

---

## 2. Rust ツールチェーンのインストール

1. **rustup の導入**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **追加コンポーネント**

   ```bash
   rustup component add clippy
   rustup component add rustfmt
   ```
3. **ツールチェーンの固定**（プロジェクトルートに `rust-toolchain.toml` を配置）

   ```toml
   [toolchain]
   channel = "stable"
   components = ["clippy", "rustfmt"]
   ```

---

## 3. プロジェクト初期化

```bash
cargo new mjlog_parser --bin
cd mjlog_parser
```

`Cargo.toml` に必要ライブラリを追加：

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
quick-xml = "0.25"       # XML パーシング用
log = "0.4"              # ロギング
env_logger = "0.10"      # ログ出力設定
anyhow = "1.0"           # エラー処理
```

---

## 4. rustfmt 設定

ルートに `rustfmt.toml` を作成し、コーディングスタイルを固定：

```toml
max_width = 100
hard_tabs = false
newline_style = "Unix"
newline_style = "Unix"
```

---

## 5. Clippy 設定

`cargo clippy` で警告をチェック。以下のように `clippy.toml` を用意すると、許可／禁止ルールを定義できます。
（任意）プロジェクトルートに `clippy.toml`:

```toml
# deny all warnings
warns = []
denies = ["clippy::all"]
allows = ["clippy::pedantic"]
```

---

## 6. Git フック／Pre-commit の自動化

1. **Git Hook スクリプト** を `.git/hooks/pre-commit` に配置:

   ```bash
   #!/usr/bin/env bash
   set -e
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```
2. 実行権を付与:

   ```bash
   chmod +x .git/hooks/pre-commit
   ```

（あるいは `pre-commit` ツールを利用して管理）

---

## 7. CI での lint/format チェック

GitHub Actions を例に `.github/workflows/ci.yml`:

```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt
      - name: fmt check
        run: cargo fmt -- --check
      - name: clippy check
        run: cargo clippy -- -D warnings
      - name: build
        run: cargo build --release
      - name: test
        run: cargo test -- --nocapture
```

---

## 8. エディタ統合

* **VSCode**: `rust-analyzer` 拡張をインストール
* 設定例 (`.vscode/settings.json`):

  ```json
  {
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true
  }
  ```

---

## 9. まとめ

1. `rustup` + `rust-toolchain.toml` でツールチェーン固定
2. `rustfmt.toml` / `clippy.toml` でスタイル・警告を管理
3. Git Hook と CI で自動チェックを徹底
4. エディタでフォーマット／補完を有効化

これで、CLAUDE CODE を使って自動生成されるコードも含め、常に一貫したフォーマットと高いコード品質を担保できる開発環境が整います。

