# TimeFormatter 詳細設計書

## メタ情報

| 項目 | 内容 |
|------|------|
| ドキュメントID | DETAIL-CLI-004-TF |
| 親設計書 | [BASIC-CLI-004_visual-enhancement.md](../../basic/BASIC-CLI-004_visual-enhancement.md) |
| 対応要件 | F-035（時間表示フォーマット改善） |
| バージョン | 1.0.0 |
| ステータス | ドラフト |
| 作成日 | 2026-01-10 |

---

## 1. 概要

### 1.1 目的

TimeFormatterは、タイマーの経過時間と目標時間を `MM:SS/MM:SS (PP%)` 形式でフォーマットする責務を持つ。ユーザーが進捗を直感的に把握できるよう、パーセンテージ表示を追加する。

### 1.2 スコープ

- 経過時間と目標時間の受け取り
- `MM:SS/MM:SS (PP%)` 形式へのフォーマット
- パーセンテージ計算（0-100%、整数）
- エッジケース処理（0除算、オーバーフロー防止）

---

## 2. アーキテクチャ

### 2.1 モジュール構成

```
src/cli/
├── display.rs       # 既存：Display構造体
├── time_format.rs   # 新規：TimeFormatter、TimeDisplay
└── mod.rs           # time_formatをpub mod
```

### 2.2 コンポーネント図

```mermaid
flowchart LR
    subgraph Display["display.rs"]
        D[Display::update_status]
    end
    
    subgraph TimeFormat["time_format.rs"]
        TF[TimeFormatter]
        TD[TimeDisplay]
    end
    
    subgraph Types["types/mod.rs"]
        TS[TimerState]
    end
    
    TS -->|remaining_seconds, duration| TF
    TF -->|TimeDisplay| TD
    TD -->|format()| D
```

---

## 3. データ型定義

### 3.1 TimeDisplay 構造体

```rust
/// 時間表示情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeDisplay {
    /// 経過時間（秒）
    pub elapsed: u64,
    /// 目標時間（秒）
    pub total: u64,
    /// パーセンテージ（0-100）
    pub percentage: u8,
}

impl TimeDisplay {
    /// 新しいTimeDisplayを作成
    /// 
    /// # Arguments
    /// * `elapsed` - 経過時間（秒）
    /// * `total` - 目標時間（秒）
    /// 
    /// # Examples
    /// ```
    /// let display = TimeDisplay::new(323, 1500);
    /// assert_eq!(display.percentage, 21);
    /// ```
    pub fn new(elapsed: u64, total: u64) -> Self {
        let percentage = if total > 0 {
            ((elapsed.saturating_mul(100)) / total).min(100) as u8
        } else {
            0
        };
        Self { elapsed, total, percentage }
    }

    /// フォーマット済み文字列を返す
    /// 
    /// # Returns
    /// `MM:SS/MM:SS (PP%)` 形式の文字列
    /// 
    /// # Examples
    /// ```
    /// let display = TimeDisplay::new(323, 1500);
    /// assert_eq!(display.format(), "05:23/25:00 (21%)");
    /// ```
    pub fn format(&self) -> String {
        let elapsed_mm = self.elapsed / 60;
        let elapsed_ss = self.elapsed % 60;
        let total_mm = self.total / 60;
        let total_ss = self.total % 60;
        
        format!(
            "{:02}:{:02}/{:02}:{:02} ({}%)",
            elapsed_mm, elapsed_ss, total_mm, total_ss, self.percentage
        )
    }
}
```

### 3.2 TimeFormatter トレイト（オプション）

```rust
/// 時間フォーマッタートレイト
/// 
/// テスト時のモック差し替えを可能にする
pub trait TimeFormatter {
    fn format_time(&self, elapsed: u64, total: u64) -> String;
}

/// デフォルト実装
pub struct DefaultTimeFormatter;

impl TimeFormatter for DefaultTimeFormatter {
    fn format_time(&self, elapsed: u64, total: u64) -> String {
        TimeDisplay::new(elapsed, total).format()
    }
}
```

---

## 4. 実装詳細

### 4.1 パーセンテージ計算ロジック

```rust
/// パーセンテージを計算（0-100の範囲に制限）
fn calculate_percentage(elapsed: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    
    // オーバーフロー防止: saturating_mul使用
    let numerator = elapsed.saturating_mul(100);
    let result = numerator / total;
    
    // 100を超えないように制限
    result.min(100) as u8
}
```

### 4.2 ビジネスルール実装

| ルールID | ルール | 実装方法 |
|---------|--------|---------|
| BR-090 | 時間表示は常に`MM:SS/MM:SS (PP%)`形式 | `format!`マクロで固定フォーマット |
| BR-091 | パーセンテージは0-100の整数 | `u8`型、`.min(100)`で上限制限 |
| BR-092 | 経過時間が目標時間を超えた場合は100% | `.min(100)`で制限 |
| BR-093 | 分・秒は2桁でゼロパディング | `{:02}`フォーマット指定子 |

### 4.3 エッジケース処理

| ケース | 入力例 | 期待出力 | 処理方法 |
|--------|--------|---------|---------|
| 目標時間0 | `(100, 0)` | `"01:40/00:00 (0%)"` | 0除算防止のif文 |
| 経過時間が目標超過 | `(2000, 1500)` | `"33:20/25:00 (100%)"` | `.min(100)`で制限 |
| 両方0 | `(0, 0)` | `"00:00/00:00 (0%)"` | 正常処理 |
| 大きな値 | `(u64::MAX, 1)` | オーバーフロー防止 | `saturating_mul`使用 |

---

## 5. 既存コードとの統合

### 5.1 display.rs への統合

```rust
// src/cli/display.rs

use crate::cli::time_format::TimeDisplay;

impl Display {
    fn create_progress_bar(
        &self,
        phase: TimerPhase,
        total_seconds: u64,
        remaining_seconds: u64,
        task_name: Option<&str>,
    ) -> ProgressBar {
        // 経過時間を計算
        let elapsed = total_seconds.saturating_sub(remaining_seconds);
        
        // TimeDisplayで時間フォーマット
        let time_display = TimeDisplay::new(elapsed, total_seconds);
        let time_str = time_display.format();
        
        // ... 既存のProgressBar作成ロジック ...
        
        // テンプレートに時間文字列を埋め込み
        let template = format!(
            "{{prefix}} [{{bar:40.{}}}] {}\n{{msg}}",
            color_code,
            time_str
        );
        
        // ...
    }
}
```

### 5.2 types/mod.rs への型追加

```rust
// src/types/mod.rs に追加

/// 時間表示情報（F-035）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeDisplay {
    pub elapsed: u64,
    pub total: u64,
    pub percentage: u8,
}
```

---

## 6. テスト設計

### 6.1 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_display_new_normal() {
        let display = TimeDisplay::new(323, 1500);
        assert_eq!(display.elapsed, 323);
        assert_eq!(display.total, 1500);
        assert_eq!(display.percentage, 21);
    }

    #[test]
    fn test_time_display_new_zero_total() {
        let display = TimeDisplay::new(100, 0);
        assert_eq!(display.percentage, 0);
    }

    #[test]
    fn test_time_display_new_overflow() {
        let display = TimeDisplay::new(2000, 1500);
        assert_eq!(display.percentage, 100);
    }

    #[test]
    fn test_time_display_format_normal() {
        let display = TimeDisplay::new(323, 1500);
        assert_eq!(display.format(), "05:23/25:00 (21%)");
    }

    #[test]
    fn test_time_display_format_zero() {
        let display = TimeDisplay::new(0, 1500);
        assert_eq!(display.format(), "00:00/25:00 (0%)");
    }

    #[test]
    fn test_time_display_format_complete() {
        let display = TimeDisplay::new(1500, 1500);
        assert_eq!(display.format(), "25:00/25:00 (100%)");
    }

    #[test]
    fn test_time_display_format_padding() {
        let display = TimeDisplay::new(65, 300);
        assert_eq!(display.format(), "01:05/05:00 (21%)");
    }

    #[test]
    fn test_percentage_boundary_values() {
        // 0%
        assert_eq!(TimeDisplay::new(0, 100).percentage, 0);
        // 50%
        assert_eq!(TimeDisplay::new(50, 100).percentage, 50);
        // 99%
        assert_eq!(TimeDisplay::new(99, 100).percentage, 99);
        // 100%
        assert_eq!(TimeDisplay::new(100, 100).percentage, 100);
        // >100% (capped)
        assert_eq!(TimeDisplay::new(150, 100).percentage, 100);
    }
}
```

### 6.2 テストカバレッジ目標

| カテゴリ | カバレッジ目標 |
|---------|---------------|
| TimeDisplay::new | 100% |
| TimeDisplay::format | 100% |
| エッジケース | 100% |
| **全体** | **100%** |

---

## 7. パフォーマンス要件

| 指標 | 目標値 | 測定方法 |
|------|--------|---------|
| format()実行時間 | 1μs以内 | `criterion`ベンチマーク |
| メモリ割り当て | 1回（String生成） | `dhat`プロファイラ |
| スタックサイズ | 24バイト（TimeDisplay構造体） | `std::mem::size_of` |

---

## 8. 変更履歴

| 日付 | バージョン | 変更内容 | 担当者 |
|:---|:---|:---|:---|
| 2026-01-10 | 1.0.0 | 初版作成 | - |
