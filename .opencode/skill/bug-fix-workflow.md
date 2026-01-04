# バグ修正完全ワークフロー

バグ発見から修正完了までの完全なライフサイクルを自動化します。

---

## 自動検出トリガー（Sisyphusが会話から判断）

| ユーザー発言パターン | 自動実行アクション |
|-------------------|------------------|
| "〇〇が動かない" "XXXのバグ" | Issue作成提案 → 承認後に修正サイクル |
| "Issue #XX を修正して" | 即座に fix/issue-XX 環境で修正開始 |
| "PRのレビュー指摘対応" | 既存環境再開 → 修正 → push |
| "-w 2が反映されない"（具体的不具合） | Issue作成 → 原因特定 → 修正 |

> **Note**: このスキルは明示的に呼び出す必要はありません。Sisyphusが会話から自動的に適用します。

---

## ワークフロー全体図

```
バグ報告
  ↓
[1. Issue確認/作成]
  ├─ 既存Issue → 取得
  └─ 未作成 → 作成提案 → ユーザー承認
  ↓
[2. 実装フェーズ] ← `/implement-issues <issue-number>` を内部で呼び出し
  ├─ container-use環境作成（fix/issue-XX-<description>）
  ├─ バグ原因特定
  ├─ 最小修正（Bugfix Rule遵守）
  ├─ Regression Test追加
  ├─ 品質レビュー（9点以上）
  └─ ユーザー承認
  ↓
[3. 完了フェーズ]
  ├─ PR作成（`Closes #XX` で自動クローズ）
  ├─ CI監視 → 通過待機
  ├─ PRマージ
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

#### 2.2 Bugfix Rule（container-worker内で遵守）

container-workerは以下のルールを遵守して修正を行う：

| ルール | 説明 |
|--------|------|
| **最小変更** | バグの根本原因のみを修正（リファクタリング禁止） |
| **Regression Test** | バグを再現するテストケースを必ず追加 |
| **原因記録** | 修正前にコメントで根本原因を記録 |
| **影響範囲確認** | 修正が他の機能に影響しないか確認 |

```python
# container-worker内での修正例
def implement_bug_fix(issue_id: int, env_id: str):
    """バグ修正実装（container-worker内で実行）"""
    
    # 1. 原因特定
    root_cause = analyze_bug(issue_id)
    
    # 2. Regression Test追加
    add_regression_test(env_id, issue_id, root_cause)
    
    # 3. 最小修正（リファクタリング禁止）
    apply_minimal_fix(env_id, root_cause)
    
    # 4. テスト実行（Regression Testが通ることを確認）
    container-use_environment_run_cmd(
        environment_id=env_id,
        command=f"cargo test test_fix_issue_{issue_id}"
    )
    
    # 5. 影響範囲確認（全テスト実行）
    container-use_environment_run_cmd(
        environment_id=env_id,
        command="cargo test"
    )
```

---

### Phase 3: 完了フェーズ

#### 3.1 PR作成（`Closes #XX` で自動クローズ）

container-workerが作成したPRには、必ず `Closes #XX` が含まれる：

```markdown
## 概要
Closes #{issue_id}

## 原因
{root_cause_description}

## 修正内容
{fix_description}

## 影響範囲
{impact_scope}

## 追加したテスト
- `test_fix_issue_{issue_id}_*`: バグ再現テスト

## チェックリスト
- [x] Bugfix Rule遵守（最小変更のみ）
- [x] Regression Test追加
- [x] 全テスト通過
- [x] 品質レビュー通過（9点以上）
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
User: "-w 2 オプションが反映されていないようです。25分のままタイマーが動作します。"

Sisyphus:
1. バグ報告を検出
2. Issue作成提案
   - タイトル: "fix: -w オプションが反映されない"
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

| 条件 | アクション |
|------|----------|
| Issue作成を拒否された | 修正を中断 |
| CI修正3回失敗 | Draft PR化、手動確認依頼 |
| PRマージ時にコンフリクト | 手動マージ依頼 |
| 環境削除3回失敗 | 手動削除依頼 |

---

## 関連ドキュメント

| ドキュメント | 参照タイミング |
|-------------|---------------|
| [/implement-issues](../command/implement-issues.md) | 実装フェーズの詳細 |
| [container-use環境構築](./container-use-guide.md) | 環境作成・管理 |
| [設計書同期ポリシー](../instructions/design-sync.md) | 設計書と実装の同期 |
| [テスト戦略](../instructions/testing-strategy.md) | Regression Test追加 |

---

## まとめ

このワークフローにより、バグ報告から修正完了までを完全自動化します。

| フェーズ | 自動化内容 |
|---------|----------|
| Issue作成 | 会話から自動検出 → 作成提案 → 承認後に作成 |
| 実装 | `/implement-issues` 内部呼び出し（Bugfix Rule遵守） |
| 完了 | PR作成 → CI監視 → マージ → クリーンナップ |

**ユーザーは「バグがある」と報告するだけで、残りは全自動で完了します。**
