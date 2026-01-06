# イベントフック機能 テスト仕様書

## 1. 概要

本ドキュメントは、イベントフック機能のテスト項目を定義する。すべてのテストは、タイマーをブロックしない非同期実行を前提とする。

## 2. テスト戦略

### 2.1 テストレベル

| テストレベル | 対象 | ツール | カバレッジ目標 |
|-------------|------|--------|---------------|
| 単体テスト | 個別関数・メソッド | cargo test | 80%以上 |
| 統合テスト | モジュール間連携 | cargo test | 70%以上 |
| E2Eテスト | 実際のスクリプト実行 | 手動テスト | 主要シナリオ |

### 2.2 テスト環境

| 環境 | OS | Rust | 用途 |
|------|-----|------|------|
| ローカル | macOS 12+ | 1.71+ | 開発時テスト |
| CI | macOS (GitHub Actions) | 1.71+ | 自動テスト |

## 3. 単体テスト

### 3.1 HookExecutor

#### 3.1.1 execute メソッド

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| UT-HE-001 | enabled=falseの場合 | enabled=false | 即座にreturn、ログなし |
| UT-HE-002 | フックが0件の場合 | hooks=[] | 即座にreturn、ログなし |
| UT-HE-003 | enabled=trueのフックのみ実行 | 2件（1件enabled=false） | 1件のみ実行 |
| UT-HE-004 | 複数フックの並列実行 | 3件のフック | 3件すべて並列実行 |
| UT-HE-005 | 無効なイベント名 | event="invalid" | フックなし、ログなし |

```rust
#[tokio::test]
async fn test_execute_disabled() {
    let executor = HookExecutor {
        config: None,
        enabled: false,
    };
    let context = HookContext::new(HookEvent::WorkEnd);
    
    let result = executor.execute(context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_parallel() {
    // 3件のフックを並列実行
    // すべてのフックが独立して実行されることを確認
}
```

#### 3.1.2 execute_single_hook 関数

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| UT-ESH-001 | スクリプト実行成功 | 正常なスクリプト | INFO: 成功ログ |
| UT-ESH-002 | スクリプト実行失敗（非0終了コード） | exit 1のスクリプト | ERROR: 失敗ログ |
| UT-ESH-003 | スクリプトタイムアウト | sleep 60のスクリプト、timeout=5 | ERROR: タイムアウトログ |
| UT-ESH-004 | スクリプト不存在 | 存在しないパス | ERROR: バリデーションエラー |
| UT-ESH-005 | 実行権限なし | chmod 644のスクリプト | ERROR: バリデーションエラー |
| UT-ESH-006 | 標準出力のログ記録 | echo "test"のスクリプト | DEBUG: 標準出力ログ |
| UT-ESH-007 | 標準エラー出力のログ記録 | echo "error" >&2のスクリプト | WARN: 標準エラー出力ログ |
| UT-ESH-008 | 10KB超過の出力 | 20KBの出力 | 10KBで切り詰め、"(truncated)" |

```rust
#[tokio::test]
async fn test_execute_single_hook_success() {
    let hook = HookDefinition {
        name: "test".to_string(),
        script: PathBuf::from("/tmp/test.sh"),
        enabled: true,
        timeout: Some(10),
    };
    
    // /tmp/test.sh を作成（echo "success"）
    // execute_single_hook を実行
    // ログに "成功しました" が含まれることを確認
}

#[tokio::test]
async fn test_execute_single_hook_timeout() {
    let hook = HookDefinition {
        name: "timeout_test".to_string(),
        script: PathBuf::from("/tmp/timeout.sh"),
        enabled: true,
        timeout: Some(2),
    };
    
    // /tmp/timeout.sh を作成（sleep 10）
    // execute_single_hook を実行
    // ログに "タイムアウトしました" が含まれることを確認
}
```

#### 3.1.3 validate_script 関数

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| UT-VS-001 | 絶対パス、存在、実行権限あり | /tmp/test.sh (chmod 755) | Ok(()) |
| UT-VS-002 | 相対パス | ./test.sh | Err("絶対パス") |
| UT-VS-003 | ファイル不存在 | /tmp/nonexistent.sh | Err("見つかりません") |
| UT-VS-004 | 実行権限なし | /tmp/test.sh (chmod 644) | Err("実行権限がありません") |

```rust
#[test]
fn test_validate_script_relative_path() {
    let path = PathBuf::from("./test.sh");
    let result = validate_script(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("絶対パス"));
}

#[test]
fn test_validate_script_no_execute_permission() {
    let path = PathBuf::from("/tmp/test.sh");
    // chmod 644 でファイル作成
    let result = validate_script(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("実行権限"));
}
```

### 3.2 HookConfig

#### 3.2.1 load メソッド

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| UT-HC-001 | ファイル不存在 | hooks.json不存在 | None、INFO: 見つかりません |
| UT-HC-002 | JSON解析エラー | 不正なJSON | None、WARN: 解析に失敗 |
| UT-HC-003 | バリデーションエラー | timeout=500 | None、WARN: バリデーションエラー |
| UT-HC-004 | 正常な設定ファイル | 正しいJSON | Some(HookConfig) |

```rust
#[test]
fn test_load_file_not_found() {
    // hooks.jsonを削除
    let config = HookConfig::load();
    assert!(config.is_none());
}

#[test]
fn test_load_invalid_json() {
    // 不正なJSONを書き込み
    let config = HookConfig::load();
    assert!(config.is_none());
}
```

#### 3.2.2 validate メソッド

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| UT-HCV-001 | global_timeout範囲外（0秒） | global_timeout=0 | Err("1-300秒") |
| UT-HCV-002 | global_timeout範囲外（301秒） | global_timeout=301 | Err("1-300秒") |
| UT-HCV-003 | 無効なイベント名 | event="invalid_event" | Err("無効なイベント名") |
| UT-HCV-004 | フック数上限超過 | 11件のフック | Err("上限（10個）を超えています") |
| UT-HCV-005 | 正常な設定 | 正しい設定 | Ok(()) |

```rust
#[test]
fn test_validate_global_timeout_out_of_range() {
    let config = HookConfig {
        hooks: HashMap::new(),
        global_timeout: 0,
    };
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("1-300秒"));
}

#[test]
fn test_validate_too_many_hooks() {
    let mut hooks = HashMap::new();
    hooks.insert("work_end".to_string(), vec![/* 11件のフック */]);
    let config = HookConfig {
        hooks,
        global_timeout: 30,
    };
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("上限"));
}
```

### 3.3 HookContext

#### 3.3.1 to_env_vars メソッド

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| UT-HCE-001 | 必須環境変数の生成 | 基本的なコンテキスト | 11個の環境変数 |
| UT-HCE-002 | タスク名なし | task_name=None | POMODORO_TASK_NAME不在 |
| UT-HCE-003 | タスク名のサニタイズ | task_name="test; rm -rf /" | エスケープされた値 |
| UT-HCE-004 | 特殊文字のエスケープ | task_name="$VAR `cmd`" | エスケープされた値 |

```rust
#[test]
fn test_to_env_vars_basic() {
    let context = HookContext {
        event: HookEvent::WorkEnd,
        phase: TimerPhase::Working,
        task_name: Some("API実装".to_string()),
        duration_secs: 1500,
        elapsed_secs: 900,
        remaining_secs: 600,
        cycle: 2,
        total_cycles: 4,
        timestamp: chrono::Utc::now(),
        session_id: uuid::Uuid::new_v4(),
    };
    
    let env_vars = context.to_env_vars();
    assert_eq!(env_vars.get("POMODORO_EVENT"), Some(&"work_end".to_string()));
    assert_eq!(env_vars.get("POMODORO_TASK_NAME"), Some(&"API実装".to_string()));
    assert_eq!(env_vars.get("POMODORO_CYCLE"), Some(&"2".to_string()));
}

#[test]
fn test_to_env_vars_sanitize() {
    let context = HookContext {
        task_name: Some("; rm -rf /".to_string()),
        // ... 他のフィールド
    };
    
    let env_vars = context.to_env_vars();
    let task_name = env_vars.get("POMODORO_TASK_NAME").unwrap();
    assert!(!task_name.contains(';'));
}
```

## 4. 統合テスト

### 4.1 タイマーエンジン連携

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| IT-TE-001 | タイマー開始時のフック発火 | start() | WorkStart発火 |
| IT-TE-002 | タイマー完了時のフック発火 | process_tick() (残り0秒) | WorkEnd発火 |
| IT-TE-003 | 一時停止時のフック発火 | pause() | Pause発火 |
| IT-TE-004 | 再開時のフック発火 | resume() | Resume発火 |
| IT-TE-005 | 停止時のフック発火 | stop() | Stop発火 |
| IT-TE-006 | フック実行中もタイマー継続 | 長時間スクリプト実行中 | タイマーは継続 |

```rust
#[tokio::test]
async fn test_timer_engine_fire_hook_on_start() {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let mut engine = TimerEngine::new(PomodoroConfig::default(), event_tx);
    
    // フック設定を作成
    // タイマー開始
    engine.start(&StartParams::default()).unwrap();
    
    // WorkStartイベントが発火されることを確認
    // フックが非同期実行されることを確認
}

#[tokio::test]
async fn test_timer_continues_during_hook_execution() {
    // 長時間実行されるフックを設定
    // タイマー開始
    // フック実行中もタイマーがカウントダウンすることを確認
}
```

### 4.2 並列実行

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| IT-PE-001 | 複数フックの並列実行 | 3件のフック | 3件すべて並列実行 |
| IT-PE-002 | フック間の独立性 | 1件失敗、2件成功 | 失敗は他に影響しない |
| IT-PE-003 | タイムアウトの独立性 | 1件タイムアウト、2件成功 | タイムアウトは他に影響しない |

```rust
#[tokio::test]
async fn test_parallel_execution() {
    // 3件のフックを設定（実行時間: 1秒、2秒、3秒）
    // すべてのフックが並列実行されることを確認
    // 総実行時間が3秒程度であることを確認（6秒ではない）
}

#[tokio::test]
async fn test_error_isolation() {
    // 3件のフックを設定（1件は失敗するスクリプト）
    // 失敗したフックが他のフックに影響しないことを確認
    // 成功したフックのログが記録されることを確認
}
```

## 5. E2Eテスト

### 5.1 実際のスクリプト実行

| テストID | テスト内容 | 手順 | 期待結果 |
|---------|-----------|------|---------|
| E2E-001 | Slack通知スクリプト | 1. hooks.json設定<br/>2. タイマー開始<br/>3. 作業完了待機 | Slackに通知が送信される |
| E2E-002 | 統計記録スクリプト | 1. hooks.json設定<br/>2. タイマー開始<br/>3. 作業完了待機 | CSVファイルに記録される |
| E2E-003 | 音楽再生スクリプト | 1. hooks.json設定<br/>2. タイマー開始<br/>3. 休憩開始待機 | 音楽が再生される |
| E2E-004 | 複数フックの同時実行 | 1. 3件のフック設定<br/>2. タイマー開始<br/>3. 作業完了待機 | 3件すべて実行される |

### 5.2 エラーケース

| テストID | テスト内容 | 手順 | 期待結果 |
|---------|-----------|------|---------|
| E2E-ERR-001 | スクリプト不存在 | 存在しないスクリプトを設定 | エラーログ、タイマーは継続 |
| E2E-ERR-002 | 実行権限なし | chmod 644のスクリプトを設定 | エラーログ、タイマーは継続 |
| E2E-ERR-003 | スクリプトタイムアウト | sleep 60のスクリプト、timeout=5 | タイムアウトログ、タイマーは継続 |
| E2E-ERR-004 | 設定ファイル破損 | 不正なJSONを設定 | 警告ログ、フック機能無効 |

## 6. 性能テスト

### 6.1 性能要件

| テストID | テスト内容 | 目標値 | 測定方法 |
|---------|-----------|--------|---------|
| PERF-001 | フック実行遅延 | 500ms以内 | イベント発生からスクリプト実行開始までの時間 |
| PERF-002 | タイマーへの影響 | 0ms（ブロックしない） | fire-and-forget方式の確認 |
| PERF-003 | 並列実行数 | 最大10個 | 10個のフックを同時実行 |

```rust
#[tokio::test]
async fn test_hook_execution_delay() {
    let start = std::time::Instant::now();
    
    // フック実行
    executor.execute(context).await.unwrap();
    
    let delay = start.elapsed();
    assert!(delay.as_millis() < 500, "フック実行遅延が500msを超えています: {:?}", delay);
}

#[tokio::test]
async fn test_timer_not_blocked() {
    // タイマー開始
    // 長時間実行されるフックを発火
    // タイマーのカウントダウンが継続することを確認
}
```

## 7. セキュリティテスト

### 7.1 セキュリティ要件

| テストID | テスト内容 | 入力 | 期待結果 |
|---------|-----------|------|---------|
| SEC-001 | 相対パスの拒否 | script="./test.sh" | エラー、フック機能無効化 |
| SEC-002 | 実行権限の確認 | chmod 644のスクリプト | エラー、該当フックをスキップ |
| SEC-003 | 環境変数のサニタイズ | task_name="; rm -rf /" | エスケープされて安全に実行 |
| SEC-004 | ログファイル権限 | ログファイル作成 | 0600（所有者のみ読み書き可） |
| SEC-005 | 設定ファイル権限警告 | chmod 644のhooks.json | 警告ログ出力、動作は継続 |

```rust
#[test]
fn test_security_relative_path() {
    let path = PathBuf::from("./malicious.sh");
    let result = validate_script(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("絶対パス"));
}

#[test]
fn test_security_env_sanitize() {
    let malicious = "; rm -rf /";
    let sanitized = sanitize_env_value(malicious);
    assert!(!sanitized.contains(';'));
    assert!(sanitized.contains("\\;"));
}
```

## 8. テストカバレッジ

### 8.1 カバレッジ目標

| モジュール | 目標カバレッジ | 測定ツール |
|-----------|---------------|-----------|
| src/hooks/executor.rs | 80%以上 | cargo-tarpaulin |
| src/hooks/config.rs | 80%以上 | cargo-tarpaulin |
| src/hooks/context.rs | 80%以上 | cargo-tarpaulin |
| 全体 | 75%以上 | cargo-tarpaulin |

### 8.2 カバレッジ測定

```bash
# カバレッジ測定
cargo tarpaulin --out Html --output-dir coverage

# カバレッジレポート確認
open coverage/index.html
```

## 9. テスト実行

### 9.1 ローカル実行

```bash
# すべてのテスト実行
cargo test

# 単体テストのみ
cargo test --lib

# 統合テストのみ
cargo test --test '*'

# 特定のテスト実行
cargo test test_execute_single_hook_success

# ログ出力付き
RUST_LOG=debug cargo test -- --nocapture
```

### 9.2 CI実行

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.71
      - run: cargo test
      - run: cargo tarpaulin --out Xml
      - uses: codecov/codecov-action@v3
```

---

## 変更履歴

| 日付 | バージョン | 変更内容 | 担当者 |
|:---|:---|:---|:---|
| 2026-01-06 | 1.0.0 | 初版作成 | - |
