# Platform Exception Policy

プラットフォーム固有コードの container-use 例外判断基準を明文化します。

---

## 概要

container-use 環境（Linux コンテナ）では、プラットフォーム固有の API を使用するコードはビルドまたは実行できません。
このポリシーでは、例外適用の判断基準と手順を定義します。

---

## 判断基準

### 例外適用条件（すべて満たす必要あり）

| 条件 | 説明 | 検証方法 |
|------|------|----------|
| ① プラットフォーム固有 API | macOS/Windows/iOS/Android 専用 API を使用 | 依存クレート・ライブラリを確認 |
| ② コンテナでビルド不可 | Linux コンテナではコンパイルエラーになる | `#[cfg(target_os = "...")]` の有無を確認 |
| ③ CI で検証可能 | GitHub Actions 等の対応ランナーで最終検証が可能 | `.github/workflows/` を確認 |

### プラットフォーム固有ライブラリ一覧

| プラットフォーム | 固有ライブラリ | 例外適用 |
|-----------------|---------------|----------|
| **macOS** | `objc2`, `cocoa`, `core-foundation`, `core-graphics`, `core-audio`, `security-framework`, `appkit`, `icrate` | ✅ 適用 |
| **Windows** | `windows-rs`, `winapi`, `win32` | ✅ 適用 |
| **iOS** | `swift`, `uikit` | ✅ 適用 |
| **Android** | `kotlin`, `android-ndk`, `jni` | ✅ 適用 |

### クロスプラットフォームライブラリ（例外不適用）

以下のライブラリはクロスプラットフォームであり、container-use 環境でビルド可能：

| ライブラリ | 説明 | 例外 |
|-----------|------|------|
| `tray-icon` | メニューバー/システムトレイ | ❌ 不適用（ビルド可能） |
| `notify-rust` | 通知 | ❌ 不適用（ビルド可能） |
| `rodio` | オーディオ再生 | ❌ 不適用（ビルド可能） |
| `image` | 画像処理 | ❌ 不適用 |
| `tokio`, `async-std` | 非同期ランタイム | ❌ 不適用 |

> **重要**: ビルド可能 ≠ 実行可能。ビルドはコンテナで行い、実行テストは CI で行う。

---

## 判断フロー

```
Issue/Subtask 受け取り
    ↓
設計書から使用ライブラリを抽出
    ↓
プラットフォーム固有ライブラリあり？
    ├─ NO → container-use 必須（通常フロー）
    └─ YES ↓
        【ビルドテスト】container-use 環境で cargo check / npm run build 実行
        ├─ 成功 → container-use 使用（テストは CI で）
        └─ 失敗 → 例外適用（ホスト環境で作業）
            ↓
        ユーザーに例外適用を報告
            ↓
        ホスト環境で実装
            ↓
        CI で最終検証（必須）
```

### ビルドテストの実施方法

事前にコンテナでビルド可能かを検証します。**推測ではなく実際にビルドを試みる**ことで、判断の正確性を保証します。

```python
def verify_container_buildable(env_id: str, language: str) -> bool:
    """コンテナ環境でビルド可能か検証"""
    
    build_commands = {
        "rust": "cargo check",
        "typescript": "npm run build",
        "python": "pip install -e . --dry-run",
    }
    
    cmd = build_commands.get(language, "echo 'Unknown language'")
    
    result = container-use_environment_run_cmd(
        environment_id=env_id,
        command=cmd
    )
    
    return result.exit_code == 0
```

**判断結果の記録**:
- ビルドテスト成功 → `container-use 使用可能` を Issue コメントに記録
- ビルドテスト失敗 → `platform-exception 適用` を Issue コメントに記録（エラーログ付き）

---

## 例外適用時のルール

### 事前報告（必須）

```markdown
## ⚠️ Platform Exception 適用

Issue #{issue_id} は macOS 固有 API (`objc2`) を使用しているため、
container-use 例外を適用し、ホスト環境で作業します。

**理由**: {specific_reason}
**最終検証**: GitHub Actions macOS ランナー
```

### コミットメッセージ

```
feat: {summary}

[platform-exception: macOS]
Closes #{issue_id}
```

### PR 本文への記載

```markdown
## ⚠️ Platform Exception

このPRはホスト環境で作成されました（container-use 非使用）。

**理由**: macOS 固有 API (`objc2`) を使用
**検証**: CI の macOS ランナーで検証
```

---

## 例外適用の具体例

### 例1: macOS 通知システム（例外適用）

```rust
// src/notification/macos.rs
use objc2::runtime::AnyObject;
use objc2_foundation::{NSString, NSUserNotification};

// → objc2 は macOS 専用、コンテナでビルド不可
// → 例外適用
```

### 例2: メニューバー UI（例外不適用）

```rust
// src/menubar/mod.rs
use tray_icon::{TrayIcon, TrayIconBuilder};
use image::RgbaImage;

// → tray-icon はクロスプラットフォーム、コンテナでビルド可能
// → 例外不適用（container-use 必須）
```

### 例3: サウンド再生（例外不適用）

```rust
// src/sound/player.rs
use rodio::{Decoder, OutputStream, Sink};

// → rodio はクロスプラットフォーム、コンテナでビルド可能
// → 例外不適用（container-use 必須）
```

---

## 条件付きコンパイルの扱い

`#[cfg(...)]` を使用した条件付きコンパイルがある場合：

| パターン | 例外適用 | 理由 |
|---------|---------|------|
| `#[cfg(target_os = "macos")]` で囲まれた部分のみ macOS 固有 | ❌ 不適用 | 他の target でビルド可能 |
| クレート全体が macOS 固有（`objc2` 等） | ✅ 適用 | コンテナでビルド不可 |
| `#[cfg(not(test))]` でテスト時は Mock | ❌ 不適用 | テストはコンテナで可能 |

### 推奨パターン: Mock 分離

```rust
// src/notification/mod.rs
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::NotificationManager;

#[cfg(not(target_os = "macos"))]
mod stub;
#[cfg(not(target_os = "macos"))]
pub use stub::NotificationManager;
```

このパターンを使用すれば、ロジック部分のテストはコンテナで実行可能。

---

## 責任分担

| 判断者 | 責任 | タイミング |
|--------|------|----------|
| **Sisyphus** | 例外適用の判断 | Issue 実装開始前 |
| **Sisyphus** | ユーザーへの報告 | 例外適用時（実装開始前） |
| **Sisyphus** | ホスト環境での実装 | 例外適用時のみ |
| **container-worker** | 例外検出の報告 | 作業中に例外が必要と判明した場合 |

### container-worker が例外を検出した場合

```python
def handle_platform_exception_detected(env_id: str, issue_id: int, reason: str):
    """container-worker が作業中に例外が必要と判明した場合"""
    
    # 1. 作業を中断
    # 2. 環境を保持（削除しない）
    # 3. Sisyphus に報告
    
    return WorkerResult(
        status="exception_required",
        env_id=env_id,
        issue_id=issue_id,
        reason=reason,
        recommendation="Sisyphus がホスト環境で実装を引き継ぐ"
    )
```

---

## 例外適用時の禁止事項

| 禁止事項 | 理由 |
|---------|------|
| CI をスキップ | プラットフォーム検証が必須 |
| 例外報告をスキップ | トレーサビリティ確保 |
| main ブランチで直接作業 | feature ブランチ必須 |
| 他の Issue と並行作業 | ブランチ競合リスク |

---

## 関連ドキュメント

| ドキュメント | 参照タイミング |
|-------------|---------------|
| [container-use.md](./container-use.md) | container-use 基本ルール |
| [testing-strategy.md](./testing-strategy.md) | 環境依存テストの Mock パターン |
| [implement-issues.md](../command/implement-issues.md) | 実装ワークフロー全体 |

---

## 変更履歴

| 日付 | バージョン | 変更内容 |
|:---|:---|:---|
| 2026-01-07 | 1.0.0 | 初版作成。プラットフォーム例外判断基準を明文化 |
