# 技術調査レポート: rodio (v0.21.x)

| 項目 | 内容 |
|------|------|
| 調査日 | 2026-01-05 |
| 調査深度 | standard |
| 対象バージョン | v0.21.1 |
| 現行バージョン | v0.20 |

## エグゼクティブサマリー

`rodio` v0.21.x は2025年7月にリリースされたメジャーアップデートを含む安定版である。内部デコーダーが `symphonia` に統一され、macOSシステムサウンドで多用される AIFF 形式のサポートが強化された。v0.20 からの移行には、ストリーム初期化 API の変更（`OutputStreamBuilder` の導入）に伴う破壊的変更への対応が必要だが、全体的なコードの簡素化とデコードパフォーマンスの向上が期待できる。

## バージョン情報

| バージョン | リリース日 | サポート状況 |
|-----------|-----------|-------------|
| v0.21.1 | 2025-07-14 | Active |
| v0.20.x | 2024-xx-xx | Maintenance |

## Breaking Changes

### 🚨 高影響

| 変更内容 | 影響範囲 | 移行方法 |
|---------|---------|---------|
| ストリームAPIの刷新 | `OutputStream::try_default()` 等 | `OutputStreamBuilder` を使用する形式に変更。または最新の `OutputStream::try_default()` の戻り値の型変更に対応する。 |
| デコーダーの挙動変更 | `Decoder::new()` | `symphonia` がデフォルトになり、`symphonia-aiff` 機能フラグを有効にすることで AIFF を直接デコード可能。 |

## 新機能・改善点

| 機能 | 概要 | 活用シーン |
|------|------|----------|
| `amplify_decibel()` | デシベル単位での音量調整 | ユーザー設定による細かい音量調整 |
| `symphonia` 統合 | 純Rust製の高性能デコーダー | macOSシステムサウンド(AIFF)の安定再生 |

## 設計への影響

### 必須対応

- [ ] `Cargo.toml` の `rodio` バージョンアップと `symphonia-aiff` feature の追加。
- [ ] `src/sound/player.rs` における `RodioSoundPlayer::new()` の初期化ロジックの修正。

### 推奨対応

- [ ] 音量調整機能の実装において `set_volume` だけでなく `amplify_decibel` の利用を検討。

## 参考リンク

- [rodio 公式リポジトリ](https://github.com/RustAudio/rodio)
- [rodio v0.21 Upgrade Guide](https://github.com/RustAudio/rodio/blob/master/UPGRADE.md)
