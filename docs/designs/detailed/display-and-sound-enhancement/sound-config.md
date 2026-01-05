# サウンド設定・再生詳細設計

## 1. 概要
macOSシステムサウンドのロードと、`rodio` v0.21 を使用した再生処理の詳細を定義する。

## 2. 実装方針
- システムサウンドパスの探索と検証
- `rodio` のストリーム管理（OutputStreamBuilder）
- AIFF デコードの検証

（詳細は `/detailed-design-workflow` で作成）
