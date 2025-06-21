Rust 設計とテストのベストプラクティス

天鳳 mjlog パーサー Rust プロジェクト向けに、設計およびテストのベストプラクティスをまとめます。

⸻

1. 設計ベストプラクティス

1.1 モジュール構成 & 名前空間
	•	機能ごとにモジュールを分割: io, parser, model, serializer, error など。
	•	公開 API を絞る: lib.rs に pub use で公開アイテムをまとめ、内部実装は mod 以下に隠蔽。

1.2 ドメインモデル設計
	•	厳密な型定義: イベントごとに構造体を定義し、enum Event でまとめる。
	•	不可変データ優先: 読み取り後はフィールドを pub で参照可能にし、ミュータブルアクセスは限定的に。

1.3 エラー処理
	•	anyhow vs thiserror: ライブラリでは thiserror でカスタム Error を定義し、アプリケーションレイヤーでは anyhow::Result を活用。
	•	Context を付与: context() メソッドでどこで失敗したか把握しやすくする。

1.4 所有権・借用
	•	所有権を明確化: 大きなバッファや文字列は &str／&[u8] 借用で扱い、高コストなコピーを回避。
	•	ライフタイム注釈: 構造体に文字列参照を埋め込む場合、ライフタイムを明示して安全性を担保。

1.5 イテレータ & ストリーミング
	•	Iterator トレイト活用: XML イベントストリームは quick_xml::Reader::events() で Iterator として処理。
	•	map / filter / collect: 中間変換を組み合わせて可読性高く。

1.6 並行性・非同期
	•	CLI は同期: シンプルなバッチ処理なら同期 I/O で十分。
	•	非同期対応: 将来 Kafka や WebAssembly 対応を想定するなら tokio + async_stream 構成を検討。

1.7 ドキュメンテーション
	•	/// でドキュメントコメント: Public API に対して豊富に記述し、cargo doc で自動生成。
	•	例示コード: README.md に Usage セクションと CLI / lib の使用例を記載。

1.8 パフォーマンスチューニング
	•	バッファサイズの調整: quick_xml::Reader::with_capacity で読み取りバッファを最適化。
	•	プロファイリング: perf や cargo flamegraph でホットパスを特定。

⸻

2. テストベストプラクティス

2.1 ユニットテスト
	•	モジュールごとに #[cfg(test)] mod tests を配置。
	•	小さな関数単位 で期待値検証。サンプル XML 断片を定義してパーサーの各メソッドを呼び出す。

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tile_id_to_string() {
        assert_eq!(id_to_tile(0), "1m");
        assert_eq!(id_to_tile(31), "white");
    }
}

2.2 統合テスト
	•	tests/ ディレクトリ にバイナリレベルのテストを置く。
	•	小さな mjlog サンプル を tests/data/ に配置し、Command 呼び出しで JSON 生成と内容検証。

#[test]
fn smoke_test_cli() {
    let output = std::process::Command::new("mjlog-parser")
        .arg("tests/data/sample.xml.gz")
        .arg("-o").arg("out.json")
        .output().unwrap();
    assert!(output.status.success());
    let json: ParserOutput = serde_json::from_str(&fs::read_to_string("out.json").unwrap()).unwrap();
    assert_eq!(json.players.len(), 4);
}

2.3 ドキュメントテスト（Doc Tests）
	•	コメント内コードブロック に簡易例を記載して自動検証。

/// ```rust
/// let xml = "<D0/>";
/// assert!(parse(xml).is_ok());
/// ```

2.4 プロパティテスト
	•	proptest クレートを利用し、任意の mjlog 断片生成 → パース → シリアライズ整合性を検証。

2.5 カバレッジ & CI
	•	cargo tarpaulin でカバレッジ測定。
	•	GitHub Actions でカバレッジレポートをプルリクにコメント。
	•	Fail on coverage drop を設定し、主要部分のテスト網羅を担保。

2.6 テストデータ管理
	•	小さく代表的なサンプル: テストケースごとに専用 mjlog 断片を用意。
	•	テスト用ユーティリティ: サンプルファイルを返す関数を tests/helpers.rs にまとめ、重複排除。

⸻

これらのベストプラクティスを採用することで、堅牢かつ保守性の高い Rust 製 mjlog パーサーを実現できます。
