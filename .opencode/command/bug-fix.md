---
description: バグ報告から修正完了までの完全ワークフロー（Issue作成→実装→PR→マージ）
argument-hint: "[Issue番号] または バグの説明"
---

# バグ修正完全ワークフロー

バグ発見から修正完了までの完全なライフサイクルを自動化します。

> **Phase規約**: {{skill:workflow-phase-convention}} を参照

---

## 自動検出トリガー（Sisyphusが会話から判断）

### トリガーパターンと優先度

| 優先度 | ユーザー発言パターン | 自動実行アクション | 検出方法 |
|-------|-------------------|------------------|---------|
| **高** | "Issue #XX を修正して" | 即座に fix/issue-XX 環境で修正開始 | 正規表現: `Issue #(\d+).*修正` |
| **高** | "PR #XX のレビュー指摘" | 既存環境再開 → 修正 → push | 正規表現: `PR #(\d+).*レビュー\|指摘` |
| **中** | "-w 2が反映されない"（具体的不具合） | Issue作成 → 原因特定 → 修正 | LLM判定（後述） |
| **低** | "〇〇が動かない" "XXXのバグ" | Issue作成提案 → 承認後に修正サイクル | LLM判定 + ユーザー確認 |

> **Note**: このスキルは明示的に呼び出す必要はありません。Sisyphusが会話から自動的に適用します。

### 自動検出ロジック（実装詳細）

```python
def detect_bug_report_from_conversation(user_message: str, conversation_history: list[str]) -> BugReport | None:
    """
    会話からバグ報告を検出
    
    Args:
        user_message: 最新のユーザーメッセージ
        conversation_history: 過去3メッセージ（文脈判定用）
    
    Returns:
        BugReport | None: バグ報告が検出された場合はBugReportオブジェクト
    """
    
    # Step 1: 明示的Issue/PR番号の検出（優先度: 高）
    issue_match = re.search(r'Issue #(\d+).*(修正|fix|直)', user_message, re.IGNORECASE)
    if issue_match:
        issue_id = int(issue_match.group(1))
        return BugReport(
            type="explicit_issue",
            issue_id=issue_id,
            confidence="high",
            skip_confirmation=True  # Issue番号明示なので確認不要
        )
    
    pr_match = re.search(r'PR #(\d+).*(レビュー|指摘|修正|対応)', user_message, re.IGNORECASE)
    if pr_match:
        pr_number = int(pr_match.group(1))
        return BugReport(
            type="pr_review_feedback",
            pr_number=pr_number,
            confidence="high",
            skip_confirmation=True
        )
    
    # Step 2: 具体的不具合報告の検出（優先度: 中）
    # パターン: "XXX が YYY ない" "XXX が YYY する（期待外動作）"
    specific_bug_patterns = [
        r'(.+)が反映されない',
        r'(.+)が動作しない',
        r'(.+)がエラーになる',
        r'(.+)が(.+)のまま',  # 例: "25分のまま"
        r'(.+)を(.+)しても(.+)ない',
    ]
    
    for pattern in specific_bug_patterns:
        match = re.search(pattern, user_message)
        if match:
            # LLM判定で誤検知を回避
            llm_result = analyze_bug_likelihood(user_message, conversation_history)
            if llm_result.is_bug and llm_result.confidence > 0.7:
                return BugReport(
                    type="specific_bug",
                    description=match.group(0),
                    observed_behavior=match.group(0),
                    confidence="medium",
                    skip_confirmation=False  # ユーザー確認必須
                )
    
    # Step 3: 一般的バグ報告の検出（優先度: 低）
    general_bug_keywords = [
        "バグ", "bug", "不具合", "おかしい", "うまくいかない",
        "動かない", "エラー", "失敗", "問題"
    ]
    
    has_bug_keyword = any(kw in user_message.lower() for kw in general_bug_keywords)
    
    if has_bug_keyword:
        # 除外パターン（誤検知回避）
        exclude_patterns = [
            # 日本語
            r'動かない.*はず',  # 設計議論
            r'動かない.*べき',  # 設計議論
            r'動かない.*思う',  # 推測
            r'バグ.*ない',     # "バグはない"
            r'調査.*動かない',  # 調査依頼（バグ修正ではない）
            r'確認.*動かない',  # 確認依頼
            # 英語
            r'(should not|shouldn\'t) work',      # 設計議論
            r'(should not|shouldn\'t) be working', # 設計議論
            r'I think.*not work',                  # 推測
            r'maybe.*not work',                    # 推測
            r'no bug',                             # "バグはない"
            r'investigate.*not work',              # 調査依頼
            r'check.*not work',                    # 確認依頼
        ]
        
        if any(re.search(pattern, user_message) for pattern in exclude_patterns):
            return None  # 除外パターンに該当
        
        # LLM判定（文脈考慮）
        llm_result = analyze_bug_likelihood(user_message, conversation_history)
        
        if llm_result.is_bug:
            return BugReport(
                type="general_bug",
                description=user_message,
                confidence="low",
                skip_confirmation=False,  # 必ずユーザー確認
                llm_analysis=llm_result
            )
    
    return None  # バグ報告なし

def analyze_bug_likelihood(message: str, context: list[str]) -> LLMAnalysisResult:
    """
    LLMを使ってバグ報告かどうかを判定（誤検知回避）
    
    Args:
        message: ユーザーメッセージ
        context: 過去3メッセージ
    
    Returns:
        LLMAnalysisResult: 判定結果（is_bug, confidence, reason）
    """
    
    prompt = f"""
以下のユーザーメッセージがバグ報告かどうかを判定してください。

# ユーザーメッセージ
{message}

# 文脈（過去3メッセージ）
{format_context(context)}

# 判定基準
- ✅ バグ報告: 「現在の動作が期待と異なる」という報告
- ❌ バグ報告でない: 設計議論、調査依頼、質問、推測

# 出力形式（JSON）
{{
  "is_bug": true/false,
  "confidence": 0.0-1.0,
  "reason": "判定理由",
  "observed_behavior": "観測された動作",
  "expected_behavior": "期待される動作"
}}
"""
    
    # LLMに判定依頼（oracle エージェント経由）
    task_id = background_task(
        agent="oracle",
        description="バグ報告の判定",
        prompt=prompt
    )
    
    response = background_output(task_id=task_id, block=True, timeout=30)
    
    # JSON形式のレスポンスをパース
    try:
        result_json = json.loads(response)
        return LLMAnalysisResult(
            is_bug=result_json.get("is_bug", False),
            confidence=result_json.get("confidence", 0.0),
            reason=result_json.get("reason", ""),
            observed_behavior=result_json.get("observed_behavior", ""),
            expected_behavior=result_json.get("expected_behavior", "")
        )
    except (json.JSONDecodeError, KeyError) as e:
        # LLM判定失敗時のフォールバック: 信頼度0で返す
        report_to_user(f"⚠️ LLM判定失敗: {e}. 保守的に判定します。")
        return LLMAnalysisResult(
            is_bug=False,
            confidence=0.0,
            reason="LLM判定失敗（フォールバック）"
        )

def confirm_bug_report_with_user(bug_report: BugReport) -> bool:
    """
    ユーザーにバグ報告の意図を確認（誤検知回避）
    
    Args:
        bug_report: 検出されたバグ報告
    
    Returns:
        bool: ユーザーが承認した場合はTrue
    """
    
    if bug_report.skip_confirmation:
        return True  # Issue番号明示の場合は確認不要
    
    confirmation_message = f"""
## 🤔 バグ報告の確認

以下の内容をバグ報告として検出しました：

- **報告内容**: {bug_report.description}
- **検出信頼度**: {bug_report.confidence}

**これはバグ修正を依頼していますか？**
- `はい`: Issue作成 → 修正開始
- `いいえ`: 通常の会話として継続
- `調査のみ`: 原因調査のみ実施（修正は保留）
"""
    
    user_response = ask_user(confirmation_message, options=["はい", "いいえ", "調査のみ"])
    
    return user_response == "はい"
```

### 誤検知リスクと回避策

| リスク | 例 | 回避策 |
|--------|-----|--------|
| 設計議論との混同 | "この機能は動かないべき" | 除外パターン: `動かない.*べき` |
| 調査依頼との混同 | "なぜ動かないか調査して" | 除外パターン: `調査.*動かない` + LLM判定 |
| 質問との混同 | "これは動かないですか？" | LLM判定で文末の疑問符を検出 |
| 推測との混同 | "動かないと思う" | 除外パターン: `動かない.*思う` |

**誤検知時のリカバリ**:
- ユーザー確認ダイアログで「いいえ」を選択 → 通常の会話として継続
- Issue作成後に「誤検知だった」と報告 → Issueをクローズ（環境は削除）

---

## ワークフロー全体図

```
バグ報告（会話から自動検出）
  ↓
[1. Issue確認/作成]
  ├─ 既存Issue → 取得
  └─ 未作成 → 作成提案 → ユーザー承認
  ↓
[2. 実装フェーズ] ← `/implement-issues <issue-number>` を内部で呼び出し
  ├─ fixブランチ作成（Sisyphus）
  ├─ container-use環境作成（fix/issue-XX-<description>）
  ├─ バグ原因特定（container-worker）
  ├─ Regression Test追加（必須）
  ├─ 最小修正（Bugfix Rule遵守）
  ├─ 全テスト実行（影響範囲確認）
  ├─ 品質レビュー（9点以上、Bugfix Rule検証含む）
  └─ ユーザー承認
  ↓
[3. 完了フェーズ]
  ├─ PR作成（`Closes #XX` で自動クローズ）
  ├─ CI監視 → 通過待機
  │   ├─ 成功 → マージへ
  │   └─ 失敗 → 修正（最大3回）→ エスカレーション
  ├─ PRマージ
  ├─ レビュー指摘対応ループ ←─┐
  │   ├─ 環境再開            │
  │   ├─ 修正実施            │
  │   ├─ push               │
  │   └─ CI再監視 ──────────┘
  └─ クリーンナップ（環境削除 + ブランチ削除）
```

---

## フェーズ詳細

### Phase 1: Issue確認/作成

#### 1.1 既存Issue確認

```python
def check_existing_issue(bug_description: str) -> int | None:
    """バグ報告に対応するIssueが既に存在するか確認"""
    
    # ユーザーが明示的にIssue番号を指定した場合
    if "#" in bug_description:
        issue_id = extract_issue_number(bug_description)
        if issue_id:
            result = bash(f"gh issue view {issue_id} --json state,title")
            if result.exit_code == 0:
                return issue_id
    
    # 類似Issueを検索（タイトル・ラベルで絞り込み）
    search_result = bash(f"""
        gh issue list --state open --label bug --limit 20 --json number,title \
        | jq '[.[] | select(.title | test("{escape_regex(bug_description)}"; "i")) | .number]'
    """)
    
    if search_result.exit_code == 0 and search_result.stdout.strip():
        candidates = json.loads(search_result.stdout)
        if candidates:
            # 候補が複数ある場合はユーザーに確認
            if len(candidates) > 1:
                return ask_user_select_issue(candidates)
            return candidates[0]
    
    return None  # 既存Issueなし
```

#### 1.2 Issue作成提案

既存Issueがない場合、ユーザーに作成を提案：

```markdown
## 🐛 バグ報告 - Issue作成提案

### 報告内容
{bug_description}

### 提案するIssue
- **タイトル**: `fix: {summary}`
- **ラベル**: `bug`
- **説明**:
  ```
  ## 現象
  {observed_behavior}
  
  ## 期待動作
  {expected_behavior}
  
  ## 再現手順
  {reproduction_steps}
  
  ## 環境
  {environment_info}
  ```

**このIssueを作成して修正を開始しますか？**
- `作成`: Issue作成 → 修正開始
- `既存利用 #XX`: 既存Issue #XX を使用
- `キャンセル`: 中断
```

#### 1.3 Issue作成実行

ユーザー承認後、Issueを作成：

```python
def create_bug_issue(bug_info: dict) -> int:
    """バグIssueを作成"""
    
    issue_body = f"""
## 現象
{bug_info['observed_behavior']}

## 期待動作
{bug_info['expected_behavior']}

## 再現手順
{bug_info.get('reproduction_steps', '（調査中）')}

## 環境
{bug_info.get('environment_info', '（調査中）')}

---
**報告者**: {bug_info.get('reporter', 'AI')}
**優先度**: {bug_info.get('priority', 'medium')}
"""
    
    result = bash(f"""
        gh issue create \
          --title "fix: {bug_info['title']}" \
          --body "{escape_body(issue_body)}" \
          --label bug
    """)
    
    if result.exit_code != 0:
        raise Exception(f"Issue作成失敗: {result.stderr}")
    
    # Issue番号を抽出
    issue_url = result.stdout.strip()
    issue_id = int(issue_url.split('/')[-1])
    
    report_to_user(f"✅ Issue #{issue_id} を作成しました: {issue_url}")
    
    return issue_id
```

---

### Phase 2: 実装フェーズ（`/implement-issues` を内部呼び出し）

バグ修正の実装フローは、既存の `/implement-issues` ワークフローと**ほぼ同じ**です。
違いは以下の点のみ：

| 項目 | Feature開発 | バグ修正 |
|------|-----------|---------|
| ブランチ名 | `feature/issue-XX-*` | `fix/issue-XX-*` |
| 修正方針 | 新規機能追加 | **最小変更**（Bugfix Rule） |
| テスト追加 | 新規テスト | **Regression Test必須** |

#### 2.1 `/implement-issues` の呼び出し

```python
def fix_bug_via_implement_issues(issue_id: int):
    """
    /implement-issues コマンドを内部で呼び出してバグ修正を実行
    
    Note: ブランチ名を fix/ にするため、事前にブランチ作成が必要
    """
    
    # Step 1: fixブランチ作成（Sisyphusが実行）
    issue = fetch_github_issue(issue_id)
    short_desc = slugify(issue.title)[:30]
    branch_name = f"fix/issue-{issue_id}-{short_desc}"
    
    bash("git checkout main && git pull origin main")
    bash(f"git checkout -b {branch_name}")
    bash(f"git push -u origin {branch_name}")
    
    # Step 2: /implement-issues を呼び出し
    # （内部的には background_task で container-worker を起動）
    task_id = background_task(
        agent="container-worker",
        description=f"Issue #{issue_id} バグ修正",
        prompt=f"""
## タスク
Issue #{issue_id} のバグを修正してください。

## ブランチ情報（Sisyphusが作成済み）
- ブランチ名: {branch_name}
- ⚠️ 新規ブランチを作成しないこと（既存を使用）
- container-use環境作成時に `from_git_ref="{branch_name}"` を指定

## バグ修正特有の要件（MUST DO）

### 1. Bugfix Rule（最小変更の原則）
- **⛔ 禁止**: 修正と同時にリファクタリングを行う
- **✅ 必須**: バグの根本原因のみを修正
- 理由: 変更範囲を最小化し、デグレードリスクを低減

### 2. Regression Test追加（必須）
- バグを再現するテストケースを追加
- 修正後にテストが通ることを確認
- テスト名: `test_fix_issue_{issue_id}_*`

### 3. 原因分析ログ
- 修正前に、バグの根本原因をコメントで記録
- PR本文に「原因」「修正内容」「影響範囲」を明記

## Issue情報
{fetch_issue_body(issue_id)}

## 期待する出力（JSON形式）
{{"issue_id": {issue_id}, "pr_number": N, "env_id": "xxx", "score": N}}
"""
    )
    
    # Step 3: 完了を待つ
    result = collect_worker_result(task_id)
    
    return result
```

#### 2.2 Bugfix Rule（実装ガイドライン）

container-workerは以下のルールを遵守して修正を行う：

| ルール | 説明 | 検証者 | 検証方法 |
|--------|------|--------|---------|
| **最小変更** | バグの根本原因のみを修正（リファクタリング禁止） | backend-reviewer | 差分行数チェック、変更範囲レビュー |
| **Regression Test** | バグを再現するテストケースを必ず追加 | container-worker | テスト実行で検証 |
| **原因記録** | 修正前にコメントで根本原因を記録 | backend-reviewer | PR本文チェック |
| **影響範囲確認** | 修正が他の機能に影響しないか確認 | container-worker | 全テスト実行 |

##### Bugfix Rule適用の責任分担

| 責任 | 実行者 | タイミング | 具体的アクション |
|------|--------|-----------|-----------------|
| **Bugfix Ruleをプロンプト化** | Sisyphus | container-worker起動時 | `fix_bug_via_implement_issues()` でMUST DOセクションに明記 |
| **ルール遵守して実装** | container-worker | 実装時 | `implement_bug_fix()` で手順に従う |
| **ルール遵守を検証** | backend-reviewer | レビュー時 | 差分行数、変更範囲、テスト追加を確認 |
| **ルール違反時の警告** | backend-reviewer | レビュー時 | スコア減点 + 指摘事項に記載 |

##### リファクタリング判定基準

| 指標 | 閾値 | 判定 |
|------|------|------|
| 変更行数 | 修正ファイルの30%以上 | ⚠️ 要レビュー |
| 変更ファイル数 | 3ファイル以上 | ⚠️ 要レビュー |
| 関数名変更 | あり | ⚠️ リファクタリングの可能性 |
| インデント変更のみ | 10行以上 | ⚠️ リファクタリングの可能性 |

**判定ロジック（backend-reviewer内）**:
```python
def check_bugfix_rule_violation(diff: str, issue_id: int) -> list[Violation]:
    """Bugfix Rule違反をチェック"""
    
    violations = []
    
    # 変更行数チェック
    changed_lines = count_changed_lines()
    total_lines = count_total_lines_in_changed_files()
    
    if changed_lines > total_lines * 0.3:
        violations.append(Violation(
            severity="warning",
            message=f"変更行数が{changed_lines}行で、ファイル全体の30%を超えています。リファクタリングが混入していないか確認してください。"
        ))
    
    # ファイル数チェック
    changed_files = count_changed_files(diff)
    if changed_files > 3:
        violations.append(Violation(
            severity="warning",
            message=f"{changed_files}ファイルが変更されています。最小変更の原則に従っているか確認してください。"
        ))
    
    # 関数名変更チェック
    if has_function_rename(diff):
        violations.append(Violation(
            severity="warning",
            message="関数名が変更されています。バグ修正に必要な変更か確認してください。"
        ))
    
    return violations

def count_changed_lines() -> int:
    """変更行数をカウント（git diff統計から）"""
    result = bash("git diff --stat HEAD")
    # 出力例: " 5 files changed, 123 insertions(+), 45 deletions(-)"
    match = re.search(r'(\d+) insertions.*?(\d+) deletions', result.stdout)
    if match:
        insertions = int(match.group(1))
        deletions = int(match.group(2))
        return insertions + deletions
    return 0

def count_total_lines_in_changed_files() -> int:
    """変更されたファイルの総行数をカウント"""
    result = bash("git diff --name-only HEAD")
    changed_files = result.stdout.strip().split('\n')
    
    total_lines = 0
    for file_path in changed_files:
        if not file_path:
            continue
        line_count_result = bash(f"wc -l {file_path}")
        # 出力例: "  123 src/main.rs"
        match = re.search(r'^\s*(\d+)', line_count_result.stdout)
        if match:
            total_lines += int(match.group(1))
    
    return total_lines

def count_changed_files() -> int:
    """変更ファイル数をカウント"""
    result = bash("git diff --name-only HEAD")
    changed_files = [f for f in result.stdout.strip().split('\n') if f]
    return len(changed_files)

def has_function_rename() -> bool:
    """関数名変更を検出"""
    result = bash("git diff HEAD --unified=0")
    
    # 関数定義パターン（複数言語対応）
    patterns = [
        r'[-+]\s*(function|def|fn|const|let|var)\s+(\w+)\s*\(',  # JavaScript/Python/Rust
        r'[-+]\s*(pub\s+)?(async\s+)?fn\s+(\w+)\s*\(',           # Rust
        r'[-+]\s*def\s+(\w+)\s*\(',                              # Python
        r'[-+]\s*function\s+(\w+)\s*\(',                         # JavaScript
    ]
    
    for pattern in patterns:
        matches = re.findall(pattern, result.stdout, re.MULTILINE)
        # 削除（-）と追加（+）のペアがあれば関数名変更
        if len(matches) >= 2:
            return True
    
    return False
```

##### Regression Test免除条件

| 条件 | 免除可否 | 代替要件 |
|------|---------|---------|
| 既存テストがバグを検出していた | ✅ 免除可 | 既存テストの修正のみ |
| タイポ修正（コメント・文字列） | ✅ 免除可 | 影響範囲が限定的であることを明記 |
| 設定値の微調整（閾値等） | ⚠️ 要相談 | 手動テスト結果をPRに記載 |
| ロジック修正 | ❌ 必須 | 例外なし |

##### container-worker内での実装フロー

```python
def implement_bug_fix(issue_id: int, env_id: str):
    """
    バグ修正実装（container-worker内で実行）
    
    フロー:
    1. 原因特定
    2. Regression Test追加（必須、免除条件を除く）
    3. 最小修正（リファクタリング禁止）
    4. Regression Test実行
    5. 影響範囲確認（全テスト実行）
    """
    
    # 1. 原因特定
    root_cause = analyze_bug(issue_id, env_id)
    report_to_user(f"🔍 原因特定: {root_cause.description}")
    
    # 2. Regression Test追加（免除条件チェック）
    if not is_regression_test_exempt(root_cause):
        add_regression_test(env_id, issue_id, root_cause)
        report_to_user(f"✅ Regression Test追加: test_fix_issue_{issue_id}_*")
    else:
        report_to_user(f"ℹ️ Regression Test免除（理由: {root_cause.exempt_reason}）")
    
    # 3. 最小修正（リファクタリング禁止）
    changed_files = apply_minimal_fix(env_id, root_cause)
    
    # 変更行数を記録（レビュー時の参考）
    diff_stats = get_diff_stats(env_id)
    report_to_user(f"📝 変更: {diff_stats.changed_lines}行（{len(changed_files)}ファイル）")
    
    # 4. Regression Test実行
    if not is_regression_test_exempt(root_cause):
        test_result = container-use_environment_run_cmd(
            environment_id=env_id,
            command=f"cargo test test_fix_issue_{issue_id}"
        )
        if test_result.exit_code != 0:
            raise TestFailureError("Regression Testが失敗しました")
    
    # 5. 影響範囲確認（全テスト実行）
    all_test_result = container-use_environment_run_cmd(
        environment_id=env_id,
        command="cargo test"
    )
    if all_test_result.exit_code != 0:
        raise TestFailureError("修正により既存テストが失敗しました（デグレード検出）")
    
    report_to_user("✅ バグ修正完了（全テスト通過）")
```

---

### Phase 3: 完了フェーズ

#### 3.1 PR作成（`Closes #XX` で自動クローズ）

container-workerが作成したPRには、バグ修正専用のテンプレートを使用：

```markdown
## 概要
Closes #{issue_id}

{バグの簡潔な説明（1行）}

## Root Cause Analysis（根本原因分析）

### 原因
{root_cause_description}

**発生条件**:
- {condition_1}
- {condition_2}

**根本原因**:
{technical_explanation}

### 修正内容
{fix_description}

**変更箇所**:
- ファイル1: {change_summary_1}
- ファイル2: {change_summary_2}

**修正行数**: {changed_lines}行（{changed_files}ファイル）

### 影響範囲
{impact_scope}

**影響を受ける機能**:
- [ ] なし（局所的な修正）
- [ ] {affected_feature_1}
- [ ] {affected_feature_2}

**デグレードリスク**: {degradation_risk_level} （低/中/高）

## Regression Test

### 追加したテスト
- `test_fix_issue_{issue_id}_*`: バグ再現テスト

**テスト内容**:
1. バグ発生条件を再現
2. 修正後の期待動作を検証

**テスト結果**:
- 修正前: ❌ 失敗（バグ再現）
- 修正後: ✅ 成功

### 全テスト結果
```
cargo test
...
test result: ok. {total_tests} passed; 0 failed; 0 ignored; 0 measured
```

## Bugfix Rule遵守チェック

- [x] 最小変更の原則（リファクタリングなし）
  - 変更行数: {changed_lines}行（ファイルの{percentage}%）
  - 判定: ✅ 30%未満
- [x] Regression Test追加
  - テスト名: `test_fix_issue_{issue_id}_*`
  - 実行結果: ✅ 通過
- [x] 原因記録
  - Root Cause Analysisセクションに記載
- [x] 影響範囲確認
  - 全テスト実行: ✅ 通過（デグレードなし）

## 品質レビュー結果

- **スコア**: {review_score}/10
- **レビュアー**: backend-reviewer
- **判定**: ✅ 合格（9点以上）

---

## ⚠️ 特記事項（該当する場合のみ）

<!-- 以下、該当する場合のみ記載 -->

### Regression Test免除理由
<!-- 
例: タイポ修正のため、既存テストで十分
-->

### 設計書との乖離
<!-- 
例: 設計書に記載のない不具合のため、実装のみ修正
修正後に設計書を更新する必要があるか: Yes/No
-->

### container-use非使用
<!-- 
⚠️ このPRはホスト環境で作成されました（Docker障害のため）
検証: CIで環境再現性を確認してください
-->
```

### PRテンプレート生成ロジック

```python
def generate_bug_fix_pr_body(fix_result: BugFixResult) -> str:
    """バグ修正専用のPR本文を生成"""
    
    template = """
## 概要
Closes #{issue_id}

{summary}

## Root Cause Analysis（根本原因分析）

### 原因
{root_cause}

**発生条件**:
{conditions}

**根本原因**:
{technical_explanation}

### 修正内容
{fix_description}

**変更箇所**:
{changed_files_summary}

**修正行数**: {changed_lines}行（{file_count}ファイル）

### 影響範囲
{impact_scope}

**影響を受ける機能**:
{affected_features}

**デグレードリスク**: {risk_level}

## Regression Test

### 追加したテスト
- `test_fix_issue_{issue_id}_*`: {test_description}

**テスト内容**:
{test_steps}

**テスト結果**:
- 修正前: ❌ 失敗（バグ再現）
- 修正後: ✅ 成功

### 全テスト結果
```
{all_test_output}
```

## Bugfix Rule遵守チェック

- [x] 最小変更の原則（リファクタリングなし）
  - 変更行数: {changed_lines}行（ファイルの{change_percentage}%）
  - 判定: {minimal_change_judgment}
- [x] Regression Test追加
  - テスト名: `test_fix_issue_{issue_id}_*`
  - 実行結果: ✅ 通過
- [x] 原因記録
  - Root Cause Analysisセクションに記載
- [x] 影響範囲確認
  - 全テスト実行: ✅ 通過（デグレードなし）

## 品質レビュー結果

- **スコア**: {review_score}/10
- **レビュアー**: {reviewer}
- **判定**: {review_judgment}

{special_notes}
"""
    
    return template.format(**fix_result.to_dict())
```

#### 3.2 CI監視 → マージ → クリーンナップ

PRマージ後の処理は `/implement-issues` と同じ：

```python
def post_pr_workflow(pr_number: int, env_id: str):
    """PR作成後: CI待機 → 成功:マージ&削除 / 失敗:修正(3回)"""
    
    # CI完了待機（最大10分）
    ci_result = wait_for_ci(pr_number, timeout=600)
    
    if ci_result == SUCCESS:
        # 自動マージ
        auto_merge_pr(pr_number, env_id)
        
        # クリーンナップ
        cleanup_environment(env_id)
        delete_remote_branch(pr_number)
        
        report_to_user(f"""
✅ バグ修正完了

- **Issue**: #{extract_issue_from_pr(pr_number)} - 自動クローズ済み
- **PR**: #{pr_number} - マージ済み
- **環境**: {env_id} - 削除済み
- **ブランチ**: 削除済み
""")
    
    elif ci_result == FAILURE:
        # CI失敗 → 修正リトライ（最大3回）
        if handle_ci_failure(pr_number, env_id):
            # 修正成功 → 再度マージ試行
            post_pr_workflow(pr_number, env_id)
        else:
            # 3回失敗 → エスカレーション
            escalate_ci_failure(pr_number, env_id)
    
    else:  # TIMEOUT
        handle_ci_timeout(pr_number, env_id)
```

#### 3.3 クリーンナップ詳細

| リソース | 削除タイミング | コマンド |
|---------|--------------|---------|
| container-use環境 | PRマージ後 | `container-use delete {env_id}` |
| リモートブランチ | PRマージ後 | `git push origin --delete fix/issue-XX-*` |
| ローカルブランチ | （オプション） | `git branch -d fix/issue-XX-*` |

```python
def cleanup_environment(env_id: str) -> bool:
    """環境削除（最大3回リトライ）"""
    for _ in range(3):
        result = bash(f"container-use delete {env_id}")
        if result.exit_code == 0:
            return True
        wait(5)
    report_to_user(f"⚠️ 環境削除失敗。手動: container-use delete {env_id}")
    return False

def delete_remote_branch(pr_number: int):
    """PRに関連するリモートブランチを削除"""
    result = bash(f"gh pr view {pr_number} --json headRefName")
    if result.exit_code != 0:
        return
    
    branch_name = json.loads(result.stdout)["headRefName"]
    bash(f"git push origin --delete {branch_name}")
```

---

## レビュー指摘対応（PRコメント対応）

PRレビューで修正依頼があった場合：

```python
def handle_pr_review_feedback(pr_number: int):
    """PRレビュー指摘に対応"""
    
    # 1. 既存環境の再利用確認
    env_id = find_environment_by_pr(pr_number)
    
    if not env_id:
        # 環境が削除されている場合、再作成
        issue_id = extract_issue_from_pr(pr_number)
        branch_name = extract_branch_from_pr(pr_number)
        
        env_id = container-use_environment_create(
            environment_source=get_repo_path(),
            title=f"PR #{pr_number} レビュー対応",
            from_git_ref=branch_name
        )
    
    # 2. 環境再開
    container-use_environment_open(
        environment_id=env_id,
        environment_source=get_repo_path()
    )
    
    # 3. 修正実施
    # （container-use環境内で修正）
    
    # 4. push
    container-use_environment_run_cmd(
        environment_id=env_id,
        command="git add . && git commit -m 'fix: レビュー指摘対応' && git push"
    )
    
    # 5. CI再監視
    post_pr_workflow(pr_number, env_id)
```

---

## ユースケース例

### 例1: 会話から自動検出

```
User: "--timeout 30 オプションが反映されていないようです。デフォルト値のまま動作します。"

Sisyphus:
1. バグ報告を検出
2. Issue作成提案
   - タイトル: "fix: --timeout オプションが反映されない"
   - ラベル: bug
3. ユーザー承認後、Issue作成
4. /implement-issues {issue_id} を内部呼び出し
5. container-use環境で修正
6. PR作成 → CI → マージ → クリーンナップ
```

### 例2: 明示的なIssue番号指定

```
User: "Issue #64 を修正してください"

Sisyphus:
1. Issue #64 を取得
2. fix/issue-64-* ブランチ作成
3. /implement-issues 64 を内部呼び出し
4. （以下同様）
```

### 例3: PRレビュー指摘対応

```
User: "PR #42 のレビュー指摘に対応してください"

Sisyphus:
1. PR #42 から Issue/環境を特定
2. 既存環境を再開（または再作成）
3. 修正実施
4. push → CI再監視
```

---

## エスカレーション条件

以下の場合、Sisyphusはユーザーに判断を仰ぐ：

| 条件 | アクション | エスカレーション内容 |
|------|----------|-------------------|
| **Issue作成を拒否された** | 修正を中断 | 「Issue作成が承認されませんでした。バグ修正を中止します。」 |
| **CI修正3回失敗** | Draft PR化、手動確認依頼 | PRをDraft化 → 失敗ログをコメント → ユーザーに報告 |
| **PRマージ時にコンフリクト** | 手動マージ依頼 | 「コンフリクト検出。手動でマージしてください: `gh pr view {pr_number}`」 |
| **環境削除3回失敗** | 手動削除依頼 | 「環境削除失敗。手動削除: `container-use delete {env_id}`」 |
| **Regression Test作成失敗** | 原因調査 → ユーザー報告 | 「バグを再現できません。再現手順を教えてください。」 |
| **設計書と実装の大幅な乖離** | `/request-design-fix` 提案 | 「設計書との乖離を検出。設計書修正を推奨します。」 |
| **container-use環境構築失敗** | Docker障害チェック → フォールバック提案 | Docker状態確認 → ホスト環境での作業を提案（要承認） |
| **品質レビュー3回失敗（9点未満）** | Draft PR作成 → ユーザー判断 | 「品質基準未達。Draft PRを作成しました。継続/中止を判断してください。」 |

### エスカレーション詳細フロー

#### 0. 品質レビュー3回失敗時

```python
def handle_quality_review_failure(issue_id: int, env_id: str, review_history: list[ReviewResult]) -> EscalationResult:
    """
    品質レビュー3回失敗時のエスカレーション
    
    失敗理由:
    - レビュースコアが9点未満
    - 致命的な問題が解決されない
    - デグレードリスクが高い
    """
    
    # 最終スコアの確認
    final_score = review_history[-1].score
    
    # Draft PR作成
    pr_number = create_draft_pr(issue_id, env_id)
    
    # レビュー履歴をコメント
    comment = f"""
## ⚠️ 品質レビュー基準未達（3回失敗）

### レビュー履歴
{format_review_history_table(review_history)}

### 最終スコア
- **スコア**: {final_score}/10
- **判定**: ❌ 不合格（9点未満）

### 主な指摘事項
{format_review_issues(review_history[-1].issues)}

---
**Draft PR作成**: #{pr_number}
環境ID: {env_id}（保持中）
"""
    
    bash(f"gh pr comment {pr_number} --body '{escape_body(comment)}'")
    
    # ユーザーに判断を仰ぐ
    escalation_message = f"""
## ⚠️ 品質レビュー基準未達（3回失敗）

Issue #{issue_id} の実装が品質基準（9点以上）に達しませんでした。

### レビュー履歴
{format_review_summary(review_history)}

### Draft PR作成済み
- **PR**: #{pr_number}
- **環境**: {env_id}（保持中）

### 次のステップ（選択してください）
1. **継続**: さらに修正を試行（最大2回）
2. **手動対応**: Draft PRを手動でレビュー・修正
3. **中止**: Issue #{issue_id} を再検討

どれを選択しますか？
"""
    
    user_choice = ask_user(escalation_message, options=["継続", "手動対応", "中止"])
    
    if user_choice == "継続":
        # 追加で2回リトライ
        return EscalationResult(status="retry", max_retries=2)
    
    elif user_choice == "中止":
        # PRをクローズ、環境削除、Issueコメント
        bash(f"gh pr close {pr_number}")
        cleanup_environment(env_id)
        close_issue_with_comment(issue_id, f"品質基準未達のため中止（最終スコア: {final_score}/10）")
        return EscalationResult(status="aborted")
    
    else:  # "手動対応"
        # 環境は保持、PRはDraftのまま
        report_to_user(f"""
✅ Draft PR #{pr_number} を手動で対応してください。

**環境情報**:
- 環境ID: {env_id}
- 確認: `container-use log {env_id}`
- コード確認: `container-use checkout {env_id}`

**修正後の手順**:
1. 環境内で修正実施
2. `git push` で更新
3. Draft解除: `gh pr ready {pr_number}`
""")
        return EscalationResult(status="pending_manual_fix", pr_number=pr_number, env_id=env_id)

def format_review_history_table(history: list[ReviewResult]) -> str:
    """レビュー履歴を表形式で整形"""
    table = "| 回数 | スコア | 判定 | 主な指摘 |\n|------|--------|------|----------|\n"
    for i, review in enumerate(history, 1):
        judgment = "✅ 合格" if review.score >= 9 else "❌ 不合格"
        main_issue = review.issues[0] if review.issues else "（なし）"
        table += f"| {i} | {review.score}/10 | {judgment} | {main_issue[:30]}... |\n"
    return table

def format_review_summary(history: list[ReviewResult]) -> str:
    """レビュー履歴のサマリー"""
    summary = ""
    for i, review in enumerate(history, 1):
        summary += f"\n**第{i}回レビュー**: {review.score}/10点\n"
        if review.issues:
            summary += f"- 指摘事項: {len(review.issues)}件\n"
    return summary
```

#### 1. Regression Test作成失敗時

```python
def handle_regression_test_failure(issue_id: int, env_id: str) -> EscalationResult:
    """
    Regression Test作成に失敗した場合のエスカレーション
    
    失敗理由:
    - バグを再現できない
    - 再現手順が不明確
    - テスト環境で再現しない
    """
    
    # 再現試行（最大3回）
    for attempt in range(3):
        try:
            reproduction_result = attempt_bug_reproduction(issue_id, env_id)
            if reproduction_result.success:
                return EscalationResult(status="resolved")
        except ReproductionError as e:
            report_to_user(f"⚠️ バグ再現失敗（{attempt + 1}/3回）: {e.reason}")
    
    # 3回失敗 → エスカレーション
    escalation_message = f"""
## ⚠️ Regression Test作成失敗

Issue #{issue_id} のバグを再現できませんでした。

### 試行内容
- 再現試行回数: 3回
- 環境: container-use {env_id}

### 次のステップ（選択してください）
1. **再現手順を提供**: より詳細な再現手順を教えてください
2. **手動テスト**: 手動テスト結果をPRに記載（Regression Test免除）
3. **修正中止**: バグが再現しないため修正を中止

どれを選択しますか？
"""
    
    user_choice = ask_user(escalation_message, options=["再現手順", "手動テスト", "中止"])
    
    if user_choice == "中止":
        cleanup_environment(env_id)
        return EscalationResult(status="aborted")
    
    return EscalationResult(status="pending_user_input")
```

#### 2. 設計書乖離検出時

```python
def handle_design_document_divergence(issue_id: int, divergence: DesignDivergence):
    """
    設計書と実装の乖離を検出した場合
    
    検出条件:
    - 修正内容が設計書に記載されていない機能に関係
    - 設計書の前提条件が実装と矛盾
    """
    
    escalation_message = f"""
## ⚠️ 設計書との乖離を検出

Issue #{issue_id} の修正中に、設計書との乖離を検出しました。

### 乖離内容
- **設計書**: {divergence.design_doc_path}
- **乖離箇所**: {divergence.section}
- **詳細**: {divergence.description}

### 推奨アクション
1. **設計書を修正**: `/request-design-fix` を実行して設計書を更新
2. **このまま続行**: 設計書は修正せず、実装のみ修正（非推奨）

どちらを選択しますか？
"""
    
    user_choice = ask_user(escalation_message, options=["設計書修正", "続行"])
    
    if user_choice == "設計書修正":
        # /request-design-fix を呼び出し
        execute_slash_command("/request-design-fix", args={
            "design_doc": divergence.design_doc_path,
            "issue_id": issue_id,
            "divergence": divergence.description
        })
```

#### 3. container-use環境構築失敗時

```python
def handle_container_use_failure(issue_id: int, error: ContainerError):
    """
    container-use環境構築に失敗した場合
    
    失敗原因:
    - Docker障害（ディスク容量、デーモン停止等）
    - container-use自体のバグ
    """
    
    # Docker状態診断
    docker_status = diagnose_docker_status()
    
    if not docker_status.is_running:
        escalation_message = f"""
## ⚠️ Docker障害を検出

container-use環境の構築に失敗しました。

### 診断結果
- Docker状態: 停止中
- 推奨: Docker Desktopを起動してください

起動後、再試行しますか？
"""
        user_choice = ask_user(escalation_message, options=["再試行", "ホスト環境で続行"])
        
        if user_choice == "再試行":
            return EscalationResult(status="retry_after_docker_restart")
    
    # ディスク容量不足
    if docker_status.disk_full:
        escalation_message = f"""
## ⚠️ ディスク容量不足

### 現在の状況
- 利用可能容量: {docker_status.available_space}
- 必要容量: 最低10GB

### 推奨アクション
1. `docker system prune -af` でクリーンアップ
2. 不要なイメージ・コンテナを削除

クリーンアップしますか？
"""
        user_choice = ask_user(escalation_message, options=["クリーンアップ", "ホスト環境で続行"])
        
        if user_choice == "クリーンアップ":
            bash("docker system prune -af")
            return EscalationResult(status="retry_after_cleanup")
    
    # フォールバック: ホスト環境での作業（要承認）
    fallback_message = f"""
## ⚠️ container-use利用不可

container-use環境が利用できません。

### オプション
1. **ホスト環境で作業**: container-use不使用（環境分離なし）
   - ⚠️ 注意: ローカル環境を汚染する可能性
   - ⚠️ 注意: PR本文に `[non-containerized]` を明記
2. **作業中止**: Docker復旧後に再試行

どちらを選択しますか？
"""
    
    user_choice = ask_user(fallback_message, options=["ホスト環境", "中止"])
    
    if user_choice == "ホスト環境":
        return EscalationResult(status="fallback_to_host", warning="[non-containerized]")
    
    return EscalationResult(status="aborted")
```

---

## プラットフォーム例外の扱い

バグ修正時も `/implement-issues` と同様、プラットフォーム固有コードの例外ルールが適用されます。

> **詳細**: [プラットフォーム例外ポリシー](../instructions/platform-exception.md) を参照

### バグ修正での適用例

| バグ内容 | 例外適用 | 理由 |
|---------|---------|------|
| ネイティブ通知が表示されない | ✅ 適用 | プラットフォーム固有 API |
| システムサウンド再生が失敗 | ✅ 適用 | OS オーディオ API |
| システムトレイ表示がおかしい | ✅ 適用 | プラットフォーム固有 UI |
| ソケット通信エラー | ❌ 不適用 | クロスプラットフォーム |
| 設定ファイル読み込み失敗 | ❌ 不適用 | クロスプラットフォーム |

## 関連ドキュメント

| スキル/ドキュメント | 参照タイミング |
|-------------------|---------------|
| {{skill:container-use-guide}} | 環境作成・管理 |
| {{skill:ci-workflow}} | PR作成後のCI監視 |
| {{skill:tdd-implementation}} | テスト追加時 |
| [プラットフォーム例外](../instructions/platform-exception.md) | 固有コードの修正時 |
| [設計書同期ポリシー](../instructions/design-sync.md) | 設計書更新時 |
| [テスト戦略](../instructions/testing-strategy.md) | Regression Test追加時 |

---

## まとめ

このワークフローにより、バグ報告から修正完了までを完全自動化します。

| フェーズ | 自動化内容 |
|---------|----------|
| Issue作成 | 会話から自動検出 → 作成提案 → 承認後に作成 |
| 実装 | `/implement-issues` 内部呼び出し（Bugfix Rule遵守） |
| 完了 | PR作成 → CI監視 → マージ → クリーンナップ |

**ユーザーは「バグがある」と報告するだけで、残りは全自動で完了します。**
