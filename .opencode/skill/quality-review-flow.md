# 品質レビューフロー & 客観的品質基準

> **参照元**: implement-issues.md から分離された品質レビューロジック

---

## 1. 品質レビュー実行フロー ⚠️ 必須

> **⚠️ 重要**: PR作成前に必ず品質レビューを実行。スキップ厳禁。

### 1.1 レビューエージェント選択

| 実装内容 | エージェント |
|----------|-------------|
| バックエンド/CLI | `backend-reviewer` |
| フロントエンド | `frontend-reviewer` |
| DB関連 | `database-reviewer` |
| インフラ | `infra-reviewer` |
| セキュリティ | `security-reviewer` |

### 1.2 スコア判定基準

| スコア | アクション |
|--------|----------|
| 9-10点 | ✅ PR作成へ |
| 7-8点 | 修正 → 再レビュー |
| 6点以下 | 設計見直し |

---

## 2. 客観的品質基準（必須条件）

レビュースコアに加え、以下の**客観的基準**を満たす必要があります。
これらはAIの主観に依存せず、ツールで検証可能です。

| 基準 | 検証コマンド | 必須 |
|------|-------------|------|
| **Lintエラー 0件** | `cargo clippy -- -D warnings` / `npm run lint` | ✅ |
| **型エラー 0件** | `cargo check` / `npm run type-check` | ✅ |
| **フォーマット準拠** | `cargo fmt --check` / `npm run format:check` | ✅ |
| **テスト全通過** | `cargo test` / `npm test` | ✅ |
| **カバレッジ 80%以上** | `cargo tarpaulin` / `npm run coverage` | 推奨 |

```python
def check_objective_criteria(env_id: str, language: str) -> ObjectiveCriteriaResult:
    """客観的品質基準のチェック"""
    
    checks = {
        "rust": {
            "lint": "cargo clippy -- -D warnings",
            "type": "cargo check",
            "format": "cargo fmt --check",
            "test": "cargo test",
        },
        "typescript": {
            "lint": "npm run lint",
            "type": "npm run type-check",
            "format": "npm run format:check",
            "test": "npm test",
        }
    }
    
    results = {}
    lang_checks = checks.get(language, {})
    
    for check_name, command in lang_checks.items():
        result = container-use_environment_run_cmd(
            environment_id=env_id,
            command=command
        )
        results[check_name] = result.exit_code == 0
    
    all_passed = all(results.values())
    
    return ObjectiveCriteriaResult(
        passed=all_passed,
        details=results,
        message="全ての客観的基準を満たしています" if all_passed else f"失敗: {[k for k, v in results.items() if not v]}"
    )
```

> **Note**: 客観的基準が未達の場合、レビュースコアに関係なく PR 作成不可。

---

## 3. 同一指摘の検出（無限ループ防止）

同じ指摘が繰り返される場合は即座にエスカレーションします。

```python
def detect_repeated_issues(current_issues: list[str], previous_issues: list[str]) -> bool:
    """前回と同じ指摘が繰り返されているか検出"""
    
    normalize = lambda s: s.lower().strip()
    current_set = set(normalize(i) for i in current_issues)
    previous_set = set(normalize(i) for i in previous_issues)
    
    overlap = current_set & previous_set
    if previous_set and len(overlap) / len(previous_set) >= 0.5:
        return True
    return False

def review_with_repeat_detection(env_id: str, subtask_id: int) -> ReviewResult:
    """同一指摘検出付きレビューループ"""
    
    MAX_RETRIES = 3
    previous_issues = []
    
    for attempt in range(MAX_RETRIES):
        # レビュー実行（各エージェント呼び出し）
        review = run_quality_review(env_id, subtask_id)
        
        # 客観的基準チェック
        objective_result = check_objective_criteria(env_id, detect_language())
        if not objective_result.passed:
             report_to_user(f"⚠️ 客観的基準未達: {objective_result.message}")
             fix_issues(env_id, ["Objective criteria failure"])
             continue

        if review.score >= 9:
            return ReviewResult(status="passed", score=review.score)
        
        # 同一指摘検出
        if attempt > 0 and detect_repeated_issues(review.issues, previous_issues):
            report_to_user(f"⚠️ 同一指摘が繰り返されています（Issue #{subtask_id}）\n前回: {previous_issues}\n今回: {review.issues}")
            return ReviewResult(status="escalated", score=review.score, reason="repeated_issues")
        
        previous_issues = review.issues
        fix_issues(env_id, review.issues)
    
    return ReviewResult(status="escalated", score=review.score, reason="max_retries")
```

---

## 4. 修正 & 再レビュー

スコア未達の場合：

1. レビュー指摘事項をTODOリストに追加
2. container-use環境内で修正を実施
3. テスト再実行で問題なしを確認
4. **再度レビューエージェントを呼び出し**（スキップ禁止）

```python
# 修正後の再レビュー呼び出し例
task(
    subagent_type="backend-reviewer",
    description="Issue #{issue_id} 修正後再レビュー",
    prompt=f"""
## 前回レビュー
- スコア: {previous_score}/10
- 指摘事項: {issues}

## 修正内容
{fix_summary}

## 再レビュー依頼
修正が適切に行われたか確認し、再スコアリングしてください。
"""
)
```

---

## 5. レビュー失敗時のエスカレーション

3回連続でスコア9点未満の場合：

1. Draft PRを作成（`--draft`フラグ）
2. PRの本文に「レビュー未通過」と明記
3. 未解決の指摘事項をPRコメントに記載
4. ユーザーに報告して判断を仰ぐ
