# Issue実装コマンド (TDD + container-use)

指定されたGitHub Issueを実装します。
**TDD（テスト駆動開発）を強制**し、品質基準を満たすまでリトライします。
**container-use環境**でクローズドな開発・テストを行います。

---

## 🚀 複数Issue並列処理（必須ルール）

> **⛔ 絶対ルール**: 複数Issue指定時は `background_task` + `container-worker` で並列処理すること。
> **`task` ツールの使用は禁止**（MCPツールが継承されずcontainer-useが使えなくなる）

複数のIssue番号が指定された場合（例: `/implement-issues 9 10`）:

### ✅ 正しい実装（必ずこうする）

```
# Step 1: 各Issueに対して background_task を呼び出す（並列）
background_task(
    agent="container-worker",
    description="Issue #9 実装",
    prompt="..."
)
background_task(
    agent="container-worker", 
    description="Issue #10 実装",
    prompt="..."
)

# Step 2: background_output で結果を収集
result_9 = background_output(task_id="...")
result_10 = background_output(task_id="...")

# Step 3: 結果をユーザーに報告
```

### ❌ 禁止パターン（絶対にしない）

| 禁止 | 理由 |
|------|------|
| `task(subagent_type="container-worker", ...)` | MCPツール（container-use）が継承されない |
| 順次処理（1つずつ実装） | 並列処理の利点が失われる |
| ホスト環境で直接実装 | container-use必須ルール違反 |

### container-worker プロンプトテンプレート

> **⚡ トークン効率**: 設計書は全文埋め込みせず、参照パスのみ渡す。

```markdown
## タスク
Issue #{issue_id} を実装し、PRを作成してください。

## リポジトリ情報
- パス: {repository_path}
- Issue詳細: `gh issue view {issue_id}` で確認

## 設計書参照
パス: `{design_doc_path}`
※ 実装に必要な部分のみ `read` ツールで参照すること

## 実装手順
1. ブランチ作成 → 2. container-use環境 → 3. TDD → 4. レビュー → 5. PR作成

## 期待する出力（JSON形式）
{"issue_id": N, "pr_number": N, "env_id": "xxx", "score": N}
```

### 並列処理ルール

| ルール | 説明 |
|--------|------|
| **1 Issue = 1 container-worker** | 各Issueは独立したエージェントで処理 |
| **1 Issue = 1 container-use環境** | 各Issueは独立した環境で実装 |
| **依存関係チェック** | 依存Issueがある場合は順次処理 |
| **結果収集** | 全エージェント完了後にサマリー報告 |

### ⛔ `task` vs `background_task` 使い分けルール（重要）

> **MCPツール（container-use）を使う必要があるエージェントを起動する場合のみ `background_task` が必須。**

| 呼び出し元 | 呼び出し先 | 使用ツール | 理由 |
|-----------|-----------|-----------|------|
| **Sisyphus** | **container-worker** | **`background_task`** | MCPツール継承が必要（⛔ `task` 禁止） |
| container-worker | backend-reviewer | `task` | MCPツール継承不要（OK） |
| container-worker | frontend-reviewer | `task` | MCPツール継承不要（OK） |

**技術的理由**:
- `task` → MCPツールが継承されない → container-workerが `container-use_*` にアクセス不可
- `background_task` → MCPツールが継承される → container-use環境での実装が可能

> **要点**: container-worker起動時は `background_task`、レビューエージェント起動時は `task` でOK

### 依存関係がある場合

```python
def implement_issues_with_deps(issue_ids: list[int]):
    """依存関係を考慮した実装"""
    
    # 依存グラフを構築
    dep_graph = build_dependency_graph(issue_ids)
    
    # 依存関係がないIssueをグループ化
    independent_groups = topological_sort(dep_graph)
    
    for group in independent_groups:
        # グループ内は並列実行
        parallel_implement(group)
        # 次グループは前グループ完了後
```

---

## ⛔ 絶対ルール（違反厳禁）

> **container-use環境の使用は必須です。ホスト環境での直接実装は一切禁止。**
> ※ 例外: プラットフォーム固有コード（後述）

| ⛔ 絶対禁止 | ✅ 必ずこうする |
|------------|----------------|
| ホスト環境で `edit` / `write` ツールを使用 | `container-use_environment_file_write` を使用 |
| ホスト環境で `bash git commit/push` を実行 | `container-use_environment_run_cmd` でgit操作 |
| ホスト環境で `bash cargo test` 等を実行 | `container-use_environment_run_cmd` でテスト |
| `cu-*` ブランチから直接PRを作成 | featureブランチを作成してからPR |
| container-use環境を作成せずに実装開始 | 必ず環境作成してから実装 |

**違反した場合**: 即座に作業を中断し、正しいフローでやり直すこと。

### 🍎 例外: プラットフォーム固有コード

以下の条件を**すべて満たす**場合のみ、ホスト環境での作業を許可:

| 条件 | 説明 |
|------|------|
| ① プラットフォーム固有API | macOS専用（objc2等）、Windows専用、iOS/Android専用 |
| ② コンテナで検証不可 | LinuxコンテナではビルドまたはAPIが利用不可 |
| ③ CI環境で検証可能 | GitHub Actions等の対応ランナーで最終検証 |

#### 判断フロー（決定木）

```python
def should_use_platform_exception(issue_id: int, design_doc: str) -> PlatformDecision:
    """
    プラットフォーム固有コード例外の判断
    
    判断者: Sisyphus（container-worker起動前に判断）
    """
    
    # 1. 設計書から使用ライブラリを抽出
    libraries = extract_libraries_from_design(design_doc)
    
    # 2. プラットフォーム固有ライブラリのチェック
    platform_specific = {
        "macos": ["objc2", "cocoa", "core-foundation", "core-graphics", 
                  "core-audio", "security-framework", "appkit"],
        "windows": ["windows-rs", "winapi", "win32"],
        "ios": ["swift", "uikit"],
        "android": ["kotlin", "android-ndk"]
    }
    
    detected_platform = None
    for platform, libs in platform_specific.items():
        if any(lib in libraries for lib in libs):
            detected_platform = platform
            break
    
    if not detected_platform:
        # プラットフォーム固有ライブラリなし → container-use必須
        return PlatformDecision(
            use_exception=False,
            reason="クロスプラットフォームコード",
            executor="container-worker"
        )
    
    # 3. コンテナでビルド可能かチェック
    can_build_in_container = check_container_compatibility(libraries)
    
    if can_build_in_container:
        # ビルドだけならコンテナで可能（実行テストはCI）
        return PlatformDecision(
            use_exception=False,
            reason="コンテナでビルド可能（実行テストはCIで実施）",
            executor="container-worker",
            ci_required=True,
            ci_runner=f"{detected_platform}-latest"
        )
    
    # 4. 例外適用
    return PlatformDecision(
        use_exception=True,
        reason=f"{detected_platform}専用APIでコンテナビルド不可",
        executor="host",  # Sisyphusがホスト環境で直接実装
        ci_required=True,
        ci_runner=f"{detected_platform}-latest"
    )
```

#### 責任分担

| 判断者 | 責任 | タイミング |
|--------|------|----------|
| **Sisyphus** | 例外適用の判断 | Issue実装開始前（container-worker起動前） |
| **Sisyphus** | ホスト環境での実装 | 例外適用時のみ |
| **container-worker** | 例外適用の報告 | 作業中に例外が必要と判明した場合 |

#### container-workerが例外を検出した場合

```python
def handle_platform_exception_in_worker(env_id: str, issue_id: int, reason: str):
    """container-worker内で例外が必要と判明した場合"""
    
    # 1. 作業を中断
    # 2. 環境を保持（削除しない）
    # 3. Sisyphusに報告して判断を委ねる
    
    return WorkerResult(
        status="exception_required",
        env_id=env_id,
        issue_id=issue_id,
        reason=reason,
        recommendation="Sisyphusがホスト環境で実装を引き継ぐ必要があります"
    )
```

**例外適用時のルール**:

```
1. 作業開始時にユーザーに例外適用を報告
2. 他のIssueとブランチ競合がないことを確認
3. featureブランチで作業（mainブランチ直接編集禁止）
4. CI通過を最終確認として必須
```

**例外に該当する例**:
- macOS: `objc2`, `cocoa`, `core-foundation`
- Windows: `windows-rs`, `winapi`
- モバイル: `swift`, `kotlin`

**例外に該当しない例**:
- クロスプラットフォームのRust/Node.js/Pythonコード → container-use必須
- 条件付きコンパイル(`#[cfg]`)でも、ロジック部分はcontainer-useで検証可能

---

## 🔀 並行作業時の環境分離（重要）

複数のIssueを並行して処理する場合、**container-use環境による分離が必須**です。

### なぜ必要か

| 問題 | ホスト環境の場合 | container-use環境の場合 |
|------|-----------------|----------------------|
| ブランチ競合 | 切り替えが必要、未コミット変更が衝突 | 各環境で独立したブランチ |
| 依存関係 | Cargo.lock/package-lock.jsonが混在 | 環境ごとに隔離 |
| ビルドキャッシュ | 互いに影響 | 完全に独立 |
| 作業中断 | 状態保持が困難 | 環境を閉じて後で再開可能 |

### 並行作業フロー

```
Issue #42 → container環境 A (env_id: abc-123)
  └─ feature/issue-42-user-auth ブランチ
  └─ 独立したファイルシステム

Issue #43 → container環境 B (env_id: def-456)
  └─ feature/issue-43-payment ブランチ
  └─ 完全に隔離された状態
```

### 環境管理

環境IDは `.opencode/environments.json` で追跡する。

#### environments.json 更新ロジック

```python
import json
from pathlib import Path

ENVIRONMENTS_FILE = ".opencode/environments.json"

def load_environments() -> dict:
    """環境情報を読み込み"""
    path = Path(ENVIRONMENTS_FILE)
    if not path.exists():
        return {"environments": []}
    return json.loads(path.read_text())

def save_environments(data: dict):
    """環境情報を保存"""
    path = Path(ENVIRONMENTS_FILE)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False))

def register_environment(issue_id: int, env_id: str, branch: str):
    """環境作成時に登録"""
    data = load_environments()
    data["environments"].append({
        "issue_id": issue_id,
        "env_id": env_id,
        "branch": branch,
        "status": "active",
        "created_at": datetime.now().isoformat(),
        "pr_number": None
    })
    save_environments(data)

def update_environment_pr(env_id: str, pr_number: int):
    """PR作成時にPR番号を記録"""
    data = load_environments()
    for env in data["environments"]:
        if env["env_id"] == env_id:
            env["pr_number"] = pr_number
            env["status"] = "pr_created"
            break
    save_environments(data)

def mark_environment_merged(env_id: str):
    """PRマージ後にステータス更新"""
    data = load_environments()
    for env in data["environments"]:
        if env["env_id"] == env_id:
            env["status"] = "merged"
            env["merged_at"] = datetime.now().isoformat()
            break
    save_environments(data)

def remove_environment(env_id: str):
    """環境削除時にレコードを削除"""
    data = load_environments()
    data["environments"] = [
        e for e in data["environments"] if e["env_id"] != env_id
    ]
    save_environments(data)

def find_environment_by_issue(issue_id: int) -> dict | None:
    """Issue IDから環境を検索（PR修正時の再利用用）"""
    data = load_environments()
    for env in data["environments"]:
        if env["issue_id"] == issue_id and env["status"] in ["active", "pr_created"]:
            return env
    return None
```

#### 更新タイミング

| イベント | 更新内容 |
|---------|---------|
| 環境作成時 | `register_environment()` で新規登録 |
| PR作成時 | `update_environment_pr()` でPR番号記録 |
| PRマージ後 | `mark_environment_merged()` でステータス更新 |
| 環境削除時 | `remove_environment()` でレコード削除 |
| PR修正時 | `find_environment_by_issue()` で既存環境を再利用 |

詳細は [container-use環境構築ガイド](../skill/container-use-guide.md) を参照。

---

## 引数

Issue番号を指定します。複数指定可能。

| 形式 | 例 | 処理方法 |
|------|-----|---------|
| 単一Issue | `/implement-issues 123` | 順次処理 **または** Subtask並列処理（自動検出） |
| 複数Issue（スペース区切り） | `/implement-issues 9 10` | **並列処理** |
| 複数Issue（カンマ区切り） | `/implement-issues 9,10,11` | **並列処理** |
| 範囲指定 | `/implement-issues 9-12` | **並列処理** (9,10,11,12) |
| 親Issue | `/implement-issues 8` | **Subtask自動検出 → 並列処理** |

### 引数パース処理

| 入力 | 出力 | 説明 |
|------|------|------|
| `123` | `[123]` | 単一Issue（Subtaskあれば展開） |
| `9 10` | `[9, 10]` | スペース区切り |
| `9,10,11` | `[9, 10, 11]` | カンマ区切り |
| `9-12` | `[9, 10, 11, 12]` | 範囲指定 |

### 🔄 親Issue → Subtask自動検出（重要）

> **単一Issue指定時は、必ずSubtaskの有無を確認すること。**

```python
def resolve_issues(issue_ids: list[int]) -> list[int]:
    """
    Issue番号リストを解決し、必要に応じてSubtaskを展開する
    
    - 単一Issue: Subtaskがあれば展開、なければそのまま
    - 複数Issue: そのまま使用（展開しない）
    """
    if len(issue_ids) == 1:
        parent_id = issue_ids[0]
        subtasks = detect_subtasks(parent_id)
        
        if subtasks:
            report_to_user(f"""
📋 親Issue #{parent_id} から {len(subtasks)}件のSubtaskを検出しました。

| Subtask | タイトル |
|---------|---------|
{format_subtask_table(subtasks)}

これらを並列実装します。
""")
            return subtasks
        else:
            # Subtaskなし → 単体実装
            return issue_ids
    else:
        # 複数指定 → そのまま使用
        return issue_ids
```

#### Subtask検出ロジック

```python
def detect_subtasks(parent_issue_id: int) -> list[int]:
    """
    親IssueからSubtaskを検出する
    
    検出パターン（優先順）:
    1. Issue bodyの "- [ ] #N" チェックリスト形式
    2. Issue bodyの "Subtask of #N" 逆参照（子→親）
    3. Issue commentsの Subtask作成記録
    
    Note: GitHub Sub-issues API (trackedInIssues) は gh CLI では取得不可のため使用しない
    """
    
    # Issue情報を取得
    result = bash(f"gh issue view {parent_issue_id} --json body,comments,number,title")
    if not result or result.exit_code != 0:
        report_to_user(f"⚠️ Issue #{parent_issue_id} の取得に失敗しました")
        return []
    
    issue_data = json.loads(result.stdout)
    subtask_ids = []
    
    # 1. Issue body からチェックリスト形式を検出
    # パターン: "- [ ] #123" or "- [x] #123" or "- #123"
    body = issue_data.get("body", "") or ""
    checkbox_patterns = [
        r"- \[[ x]\] #(\d+)",      # チェックボックス形式
        r"- #(\d+)",                # シンプルなリスト形式
        r"\* #(\d+)",               # アスタリスク形式
    ]
    for pattern in checkbox_patterns:
        matches = re.findall(pattern, body)
        subtask_ids.extend([int(m) for m in matches])
    
    if subtask_ids:
        return list(set(subtask_ids))
    
    # 2. Comments から Subtask作成記録を検出
    # /decompose-issue が作成するコメント形式を検出
    comments = issue_data.get("comments", []) or []
    for comment in comments:
        comment_body = comment.get("body", "") or ""
        
        # 検出パターン: "Created subtask #N", "Subtask #N", "Sub-issue #N"
        if any(kw in comment_body for kw in ["Subtask", "subtask", "Sub-issue", "Created #"]):
            matches = re.findall(r"#(\d+)", comment_body)
            # 親Issue自身を除外
            subtask_ids.extend([
                int(m) for m in matches 
                if int(m) != parent_issue_id
            ])
    
    # 3. 逆参照検索（子Issueが "Subtask of #N" を持つ場合）
    if not subtask_ids:
        # リポジトリ内のOpen Issueを検索
        search_result = bash(f'''
            gh issue list --state all --limit 100 --json number,body \
            | jq '[.[] | select(.body != null) | select(.body | test("Subtask of #{parent_issue_id}|Parent: #{parent_issue_id}")) | .number]'
        ''')
        if search_result.exit_code == 0 and search_result.stdout.strip():
            try:
                found_ids = json.loads(search_result.stdout)
                subtask_ids.extend(found_ids)
            except json.JSONDecodeError:
                pass
    
    return list(set(subtask_ids))  # 重複排除
```

#### 検出失敗時のフォールバック

```python
def detect_subtasks_with_fallback(parent_issue_id: int) -> tuple[list[int], str]:
    """
    Subtask検出（検出方法も返す）
    
    Returns:
        (subtask_ids, detection_method)
    """
    subtasks = detect_subtasks(parent_issue_id)
    
    if subtasks:
        return (subtasks, "auto_detected")
    
    # 検出できなかった場合、ユーザーに確認
    # Issue自体がSubtaskを持つ設計かどうか不明なため
    return ([], "none_found")
```

#### 検出結果に応じた処理フロー

| 検出結果 | 処理 |
|---------|------|
| Subtask検出（N件） | 依存関係チェック → 並列/順次実装 |
| Subtaskなし + 200行以下 | 単体実装 |
| Subtaskなし + 200行超 | `/decompose-issue` を案内 |

#### Subtask依存関係チェック（並列実行前に必須）

> **⚠️ 重要**: Subtask間に依存関係がある場合、並列実行すると失敗する可能性がある。
> 必ず依存関係をチェックし、依存がある場合は順序付けて実行すること。

```python
def check_subtask_dependencies(subtask_ids: list[int]) -> list[list[int]]:
    """
    Subtask間の依存関係をチェックし、実行グループに分割
    
    Returns:
        実行グループのリスト（各グループ内は並列実行可能）
        例: [[9, 10], [11]]  # 9,10を並列実行 → 完了後に11を実行
    """
    dependencies = {}  # {issue_id: [depends_on_ids]}
    
    for issue_id in subtask_ids:
        result = bash(f"gh issue view {issue_id} --json body,title")
        issue_data = json.loads(result.stdout)
        body = issue_data.get("body", "") or ""
        
        # 依存関係パターンを検出
        # "Depends on #N", "Blocked by #N", "After #N", "Requires #N"
        dep_patterns = [
            r"[Dd]epends on #(\d+)",
            r"[Bb]locked by #(\d+)",
            r"[Aa]fter #(\d+)",
            r"[Rr]equires #(\d+)",
        ]
        
        deps = []
        for pattern in dep_patterns:
            matches = re.findall(pattern, body)
            deps.extend([int(m) for m in matches if int(m) in subtask_ids])
        
        dependencies[issue_id] = list(set(deps))
    
    # トポロジカルソートで実行順序を決定
    return topological_sort_groups(subtask_ids, dependencies)

def topological_sort_groups(ids: list[int], deps: dict[int, list[int]]) -> list[list[int]]:
    """
    依存関係を考慮してグループ分け
    
    例:
    - #9: 依存なし
    - #10: 依存なし
    - #11: #9に依存
    
    結果: [[9, 10], [11]]
    """
    # 入次数を計算
    in_degree = {id: 0 for id in ids}
    for id, dep_list in deps.items():
        for dep in dep_list:
            if dep in in_degree:
                in_degree[id] += 1
    
    groups = []
    remaining = set(ids)
    
    while remaining:
        # 入次数0のノードを現在のグループに
        current_group = [id for id in remaining if in_degree.get(id, 0) == 0]
        
        if not current_group:
            # 循環依存を検出
            raise ValueError(f"循環依存を検出: {remaining}")
        
        groups.append(current_group)
        
        # 処理済みノードを除外し、依存先の入次数を減らす
        for id in current_group:
            remaining.remove(id)
            for other_id in remaining:
                if id in deps.get(other_id, []):
                    in_degree[other_id] -= 1
    
    return groups
```

#### 依存関係に応じた実行フロー

```python
def implement_subtasks_with_deps(parent_id: int, subtask_ids: list[int]):
    """依存関係を考慮したSubtask実装"""
    
    # 依存関係をチェック
    execution_groups = check_subtask_dependencies(subtask_ids)
    
    if len(execution_groups) == 1:
        # 依存関係なし → 全て並列実行
        report_to_user(f"📋 {len(subtask_ids)}件のSubtaskを並列実装します")
        parallel_implement(execution_groups[0])
    else:
        # 依存関係あり → グループごとに順次実行
        report_to_user(f"""
📋 {len(subtask_ids)}件のSubtaskを依存関係に従って実装します

| グループ | Subtask | 依存先 |
|---------|---------|--------|
{format_dependency_table(execution_groups)}

依存関係があるため、グループ単位で順次実行します。
""")
        
        for i, group in enumerate(execution_groups):
            report_to_user(f"🔄 グループ {i+1}/{len(execution_groups)} を実行中...")
            results = parallel_implement(group)
            
            # グループ内で失敗があれば中断
            if any(r['status'] == 'failed' for r in results):
                report_to_user("⚠️ 依存元の実装に失敗。後続グループをスキップします")
                break
```

| 依存パターン | 検出キーワード |
|-------------|---------------|
| 明示的依存 | `Depends on #N`, `Blocked by #N` |
| 順序指定 | `After #N`, `Requires #N` |
| 暗黙的依存 | （検出不可 → 失敗時に報告） |

## ワークフロー概要

<!-- [DIAGRAM-FOR-HUMANS] 全体ワークフロー図（AI処理時はスキップ）
単一Issue指定 → Subtask検出 → [Subtaskあり] → 並列実装
                           → [Subtaskなし] → 粒度チェック → [200行超] → /decompose-issue
                                                        → [200行以下] → 単体実装

Issue（200行以下） → ブランチ作成 → container-use環境 → TDD → レビュー → PR作成 → CI → マージ
→ 全Subtask完了 → Parent Issue Close
-->

## 🔄 前提条件: 適切な粒度のIssue

> **⛔ 重要**: `/implement-issues` は**200行以下のIssue**を対象とする。
> 大きなIssueは事前に分解してから実行すること。

### Issue粒度の判定

| 粒度 | 対応コマンド |
|------|-------------|
| **200行以下** | → `/implement-issues` で直接実装 |
| **200行超** | → `/decompose-issue` で分割してから実装 |
| **新規設計** | → `/detailed-design-workflow` で設計時に適切な粒度でIssue作成 |

### 粒度ルール

| 項目 | 基準 |
|------|------|
| **コード量** | 200行以下 |
| **ファイル数** | 1-3ファイル |
| **責務** | 単一責務（1つの機能・1つの目的） |
| **テスト可能性** | 独立してテスト可能 |
| **所要時間目安** | 10-15分で実装完了 |

### コード行数の見積もり方法

```python
def estimate_code_lines(issue_id: int) -> int:
    """
    Issueの実装コード行数を見積もる
    
    見積もり方法（優先順）:
    1. Issue labelsから推定（推奨）
    2. 設計書から推定
    3. Issueタイトル・本文から推定
    """
    
    # 1. Labelsから推定（最も信頼性が高い）
    result = bash(f"gh issue view {issue_id} --json labels")
    labels = json.loads(result.stdout).get("labels", [])
    label_names = [l["name"] for l in labels]
    
    # サイズラベルがあれば使用
    size_map = {
        "size/xs": 50,      # ~50行
        "size/s": 100,      # ~100行
        "size/m": 200,      # ~200行（境界）
        "size/l": 400,      # ~400行（要分割）
        "size/xl": 800,     # ~800行（要分割）
    }
    for label, lines in size_map.items():
        if label in label_names:
            return lines
    
    # 2. 設計書から推定
    design_doc = find_related_design_doc(issue_id)
    if design_doc:
        # 設計書のコードブロック行数をカウント
        code_blocks = extract_code_blocks(design_doc)
        estimated = sum(len(block.split('\n')) for block in code_blocks)
        if estimated > 0:
            return estimated * 1.5  # バッファ込み
    
    # 3. Issue本文から推定（フォールバック）
    result = bash(f"gh issue view {issue_id} --json body,title")
    issue_data = json.loads(result.stdout)
    
    # キーワードベースの推定
    body = (issue_data.get("body") or "").lower()
    title = (issue_data.get("title") or "").lower()
    
    # 複雑さ指標
    complexity_keywords = {
        "refactor": 150,
        "add": 100,
        "fix": 50,
        "update": 80,
        "implement": 200,
        "create": 150,
    }
    
    for keyword, lines in complexity_keywords.items():
        if keyword in title or keyword in body:
            return lines
    
    # デフォルト: 不明な場合は150行と仮定
    return 150

def should_decompose(issue_id: int) -> bool:
    """分割が必要かどうか判定"""
    estimated = estimate_code_lines(issue_id)
    return estimated > 200
```

#### サイズラベルの推奨

プロジェクトで以下のラベルを使用することを推奨:

| ラベル | 目安行数 | 対応 |
|--------|---------|------|
| `size/xs` | ~50行 | 直接実装 |
| `size/s` | ~100行 | 直接実装 |
| `size/m` | ~200行 | 直接実装（境界） |
| `size/l` | ~400行 | **要分割** |
| `size/xl` | ~800行以上 | **要分割** |

> **Tip**: `/decompose-issue` 実行時にサイズラベルを自動付与すると、見積もり精度が向上する。

### 大きなIssueを見つけた場合

```bash
# 1. まず分解コマンドを実行
/decompose-issue 8

# 2. 作成されたSubtaskを実装
/implement-issues 25 26 27
```

### リトライポリシー

| 条件 | アクション |
|------|----------|
| Issue失敗（1-2回目） | 同一環境でリトライ |
| Issue失敗（3回目） | Draft PRを作成、ユーザーに報告 |
| 複数Issue並列時 | 失敗したIssueのみ報告、他は継続 |

## 実行プロセス

### 0. ブランチ作成 (container-use環境作成前) ⚠️ 必須

Issue着手時に、まず**featureブランチを作成**します。

> **⚠️ 重要**: container-use環境が作成する `cu-*` ブランチを直接PRに使用してはいけません。
> 必ずfeatureブランチを作成し、そのブランチで作業を行ってください。

#### 責任者: Sisyphus（親エージェント）

> **⛔ 絶対ルール**: ブランチ作成は**必ずSisyphus**が行う。container-workerはブランチを作成しない。

| 処理 | 実行者 | 理由 |
|------|--------|------|
| ブランチ作成 | **Sisyphus** | ホスト環境でのgit操作 |
| container-use環境作成 | container-worker | 作成済みブランチを`from_git_ref`で指定 |

#### 単体実装時

```python
# Sisyphus がホスト側でブランチ作成 (bashツール使用)
bash("git checkout main && git pull origin main")
bash(f"git checkout -b feature/issue-{issue_id}-{short_description}")
bash(f"git push -u origin feature/issue-{issue_id}-{short_description}")

# その後 container-worker を起動
background_task(
    agent="container-worker",
    prompt=f"""
    ## ブランチ情報（Sisyphusが作成済み）
    - ブランチ名: feature/issue-{issue_id}-{short_description}
    - from_git_ref でこのブランチを指定してcontainer-use環境を作成すること
    ...
    """
)
```

#### 並列実装時（複数Issue）

```python
def prepare_branches_for_parallel(issue_ids: list[int]) -> dict[int, str]:
    """
    Sisyphusが全Issueのブランチを事前に作成
    
    Returns:
        {issue_id: branch_name} のマッピング
    """
    branches = {}
    
    # mainを最新化（1回だけ）
    bash("git checkout main && git pull origin main")
    
    for issue_id in issue_ids:
        issue = fetch_github_issue(issue_id)
        short_desc = slugify(issue.title)[:30]
        branch_name = f"feature/issue-{issue_id}-{short_desc}"
        
        # ブランチ作成 & プッシュ
        bash(f"git checkout main")  # 毎回mainから分岐
        bash(f"git checkout -b {branch_name}")
        bash(f"git push -u origin {branch_name}")
        
        branches[issue_id] = branch_name
    
    # mainに戻る
    bash("git checkout main")
    
    return branches

# 使用例
branches = prepare_branches_for_parallel([9, 10, 11])

# 各container-workerにブランチ名を渡す
for issue_id, branch_name in branches.items():
    background_task(
        agent="container-worker",
        prompt=f"""
        ## ブランチ情報（Sisyphusが作成済み）
        - ブランチ名: {branch_name}
        - ⚠️ 新規ブランチを作成しないこと（既存を使用）
        ...
        """
    )
```

**ブランチ命名規則**:
| プレフィックス | 用途 |
|---------------|------|
| `feature/issue-{N}-*` | 機能追加 |
| `fix/issue-{N}-*` | バグ修正 |
| `refactor/issue-{N}-*` | リファクタリング |

**アンチパターン（禁止事項）**:
| ❌ 禁止 | ✅ 正しい方法 |
|--------|-------------|
| `cu-*` ブランチから直接PRを作成 | featureブランチからPRを作成 |
| container-workerがブランチを作成 | Sisyphusが事前にブランチを作成 |
| ブランチ作成をスキップしてcontainer-use環境を開始 | 先にfeatureブランチを作成してからcontainer-use環境を作成 |
| ホスト環境で `edit`/`write` ツールを使ってコード編集 | `container-use_environment_file_write` を使用 |
| ホスト環境で `bash` ツールを使ってテスト実行 | `container-use_environment_run_cmd` を使用 |
| container-use環境なしで実装を開始 | 必ず環境作成後に実装開始 |

### 0.5. 設計書存在チェック ⚠️ 必須

> **⚠️ 重要**: 実装開始前に、対象Issueに対応する詳細設計書が存在することを確認してください。

```python
def check_design_document(issue_id: int) -> DesignDocResult:
    """
    Issueに対応する設計書の存在を確認
    
    Returns:
        DesignDocResult: 設計書の存在状態と参照パス
    """
    
    # 1. Issueからラベル・タイトルを取得
    issue = fetch_github_issue(issue_id)
    
    # 2. 詳細設計書ディレクトリを検索
    design_dirs = glob("docs/designs/detailed/**/")
    
    # 3. 関連する設計書を特定
    related_docs = find_related_design_docs(issue, design_dirs)
    
    if not related_docs:
        return DesignDocResult(
            exists=False,
            warning="⚠️ 詳細設計書が見つかりません",
            recommendation="設計書作成を先に行うか、ユーザーに確認してください"
        )
    
    return DesignDocResult(
        exists=True,
        paths=related_docs,
        message=f"✅ 設計書確認: {len(related_docs)}件"
    )
```

#### 設計書が存在しない場合

| 状況 | アクション |
|------|----------|
| 設計書なし + 小規模変更 | ユーザーに確認 → 承認されれば続行 |
| 設計書なし + 大規模変更 | 実装中断 → 詳細設計ワークフロー実行を推奨 |
| 設計書あり | 通常フローで続行 |

```python
# 設計書確認の実装例
design_result = check_design_document(issue_id)

if not design_result.exists:
    # ユーザーに確認
    user_response = ask_user(f"""
⚠️ Issue #{issue_id} に対応する詳細設計書が見つかりません。

**推奨アクション**:
- 大規模な機能追加の場合: `/detailed-design-workflow` を先に実行
- 小規模な修正の場合: このまま続行可能

このまま実装を続行しますか？ (続行/中断)
""")
    
    if user_response != '続行':
        abort_with_message("設計書作成後に再実行してください")
```

### 0.6. 設計書参照ルール（トークン最適化）⚠️ 必須

> **⛔ 絶対禁止**: 設計書の全文読み込み
> **✅ 必須**: Subtaskに必要なセクションのみ参照

#### 読み取り可能なセクション（ホワイトリスト）

| Subtask内容 | 読むべきセクション | 読んではいけない |
|------------|------------------|----------------|
| **型定義** | `## データ型`, `## インターフェース` | 画面設計、テスト仕様 |
| **API実装** | `## エンドポイント`, `## リクエスト/レスポンス` | UI、インフラ |
| **UI実装** | `## 画面仕様`, `## コンポーネント` | バックエンド、DB |
| **テスト** | `## テストケース`, `## 境界条件` | 実装詳細 |

#### 実装例

```python
def read_design_for_subtask(design_doc_path: str, subtask_type: str) -> str:
    """Subtaskに必要なセクションのみ読み取る"""
    
    # セクション別の読み取りルール
    section_map = {
        "type_definition": ["## データ型", "## インターフェース", "## 型定義"],
        "api_implementation": ["## エンドポイント", "## API", "## リクエスト"],
        "ui_implementation": ["## 画面仕様", "## コンポーネント", "## UI"],
        "test_implementation": ["## テストケース", "## テスト仕様"],
    }
    
    allowed_sections = section_map.get(subtask_type, [])
    
    # 設計書をセクション単位で読み取り
    content = read_sections_only(design_doc_path, allowed_sections)
    
    # トークン数チェック（2000トークン上限）
    if estimate_tokens(content) > 2000:
        content = summarize_to_limit(content, max_tokens=2000)
    
    return content
```

#### トークン予算

| 項目 | 上限 |
|------|------|
| 設計書参照（1 Subtask） | 2,000 トークン |
| Subtask Issue本文 | 500 トークン |
| コードコンテキスト（レビュー時） | 3,000 トークン |
| **合計（1 Subtask）** | **~6,000 トークン** |

> **比較**: 従来は1 Issueで30,000トークン消費 → Subtask方式で1/5に削減

### 1. container-use環境構築

**`from_git_ref`でfeatureブランチを指定**して環境を作成します。

```python
# 環境作成 (featureブランチから)
container-use_environment_create(
    environment_source="/path/to/repo",
    title=f"Issue #{issue_id} - {issue_title}",
    from_git_ref=f"feature/issue-{issue_id}-{short_description}"
)
```

これにより:
- featureブランチのコードがcontainer内にチェックアウトされる
- mainブランチは影響を受けない
- container内での変更はfeatureブランチにコミットされる

#### 1.1 環境設定

```python
container-use_environment_config(
    environment_id=env_id,
    environment_source="/path/to/repo",
    config={
        "base_image": "node:20-slim",
        "setup_commands": [
            "npm ci",
            "npm run build"
        ],
        "envs": [
            "NODE_ENV=test",
            "DATABASE_URL=postgresql://app:password@db:5432/testdb"
        ]
    }
)
```

#### 1.2 サービス追加 (必要に応じて)

```python
# PostgreSQL
container-use_environment_add_service(
    environment_id=env_id,
    environment_source="/path/to/repo",
    name="db",
    image="postgres:15",
    envs=["POSTGRES_USER=app", "POSTGRES_PASSWORD=password", "POSTGRES_DB=testdb"],
    ports=[5432]
)

# Redis (必要な場合)
container-use_environment_add_service(
    environment_id=env_id,
    environment_source="/path/to/repo",
    name="redis",
    image="redis:7-alpine",
    ports=[6379]
)
```

### 2. 申し送り確認 (Handover)

Issueのコメントをスキャンし、未完了の申し送り事項があれば最優先で対応。

### 3. TDD実装 (Red -> Green -> Refactor)

**全てcontainer-use環境内で実行**:

#### 3.0 テスト項目書の参照（推奨）

TDD開始前に、詳細設計フェーズで作成されたテスト項目書を参照する。

```python
def get_test_specification(issue_id: int, design_doc_path: str) -> TestSpec | None:
    """テスト項目書からテストケースを取得"""
    
    # テスト項目書のパスを推定
    design_dir = Path(design_doc_path).parent
    test_spec_path = design_dir.parent / "test-specification.md"
    
    if not test_spec_path.exists():
        # テスト項目書がない場合は設計書から推論
        return None
    
    # テスト項目書から該当Subtaskのテストケースを抽出
    test_spec = read(test_spec_path)
    relevant_cases = extract_test_cases_for_subtask(test_spec, issue_id)
    
    return TestSpec(
        cases=relevant_cases,
        boundary_conditions=extract_boundary_conditions(relevant_cases),
        error_scenarios=extract_error_scenarios(relevant_cases)
    )
```

**テスト項目書活用のメリット**:
- 詳細設計フェーズで網羅性が検証済み
- 境界条件・エラーケースが明確
- TDDのRed→Greenがスムーズに

**テスト項目書がない場合**:
- 設計書から必要なテストケースを推論
- 基本的なハッピーパス + エラーケースを実装

#### 🔴 Red: テスト実装

```python
# テスト実行 (失敗を確認)
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="npm test -- --testPathPattern='feature-name'"
)
```

#### 🟢 Green: 最小実装

```python
# ファイル編集
container-use_environment_file_write(
    environment_id=env_id,
    environment_source="/path/to/repo",
    target_file="src/feature.ts",
    contents="// implementation"
)

# テスト実行 (成功を確認)
container-use_environment_run_cmd(...)
```

#### 🔵 Refactor: 整理

```python
# Lint & 型チェック
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="npm run lint -- --fix && npm run type-check"
)
```

### 4. DBマイグレーションのテスト (DB関連Issue)

```python
# マイグレーション実行
container-use_environment_run_cmd(command="npx flyway migrate")

# ロールバックテスト
container-use_environment_run_cmd(command="npx flyway undo")

# 再マイグレーション
container-use_environment_run_cmd(command="npx flyway migrate")
```

### 5. 設計不備への対応

設計の矛盾が見つかった場合は `/request-design-fix` を実行。

### 6. 申し送り作成

他領域への影響がある場合は [申し送り処理ガイド](../skill/handover-process.md) に従う。

### 7. 品質レビュー ⚠️ 必須

> **⚠️ 重要**: PR作成前に必ず品質レビューを実行すること。スキップ厳禁。

#### 7.1 レビュー対象の確認

実装完了後、以下を確認してからレビューを依頼：

```python
# Lint & 型チェック通過を確認
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="cargo clippy -- -D warnings && cargo fmt --check"  # Rust
    # command="npm run lint && npm run type-check"  # TypeScript
)

# テスト全通過を確認
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command="cargo test"  # Rust
    # command="npm test"  # TypeScript
)
```

#### 7.2 レビューエージェント選択

| 実装内容 | 使用エージェント |
|----------|------------------|
| バックエンド/ライブラリ/CLI | `backend-reviewer` |
| フロントエンドUI | `frontend-reviewer` |
| データベース関連 | `database-reviewer` |
| インフラ/CI/CD | `infra-reviewer` |
| セキュリティ関連 | `security-reviewer` |

複数領域にまたがる場合は、主要な領域のレビューエージェントを使用。

#### 7.3 レビュー実行

**`task` を使用してレビューエージェントを呼び出す**（✅ OK - レビューエージェントはMCPツール不要）：

```python
# backend-reviewer の例（container-worker内またはSisyphusから呼び出し）
task(
    subagent_type="backend-reviewer",
    description="Issue #{issue_id} 実装コードレビュー",
    prompt=f"""
## レビュー対象
- Issue: #{issue_id} - {issue_title}
- 変更ファイル: {changed_files}
- 設計書: {design_doc_path}

## レビュー依頼
以下の観点でコードをレビューし、10点満点でスコアリングしてください：

1. **設計書との整合性** - 詳細設計書の仕様を正しく実装しているか
2. **コード品質** - SOLID原則、命名規則、可読性
3. **エラーハンドリング** - 適切なエラー処理、境界条件の考慮
4. **テスト** - カバレッジ、エッジケースの網羅
5. **セキュリティ** - 脆弱性、入力検証

## 出力形式
- **総合スコア**: X/10
- **問題点**: （あれば具体的に）
- **改善提案**: （あれば具体的に）
"""
)
```

#### 7.4 スコア判定

| スコア | アクション |
|--------|----------|
| **9点以上** | ✅ レビュー通過 → コミット & PR作成へ |
| **7-8点** | ⚠️ 指摘事項を修正 → 再レビュー |
| **6点以下** | ❌ 重大な問題あり → 設計見直しを検討 |

#### 7.5 修正 & 再レビュー

スコア未達の場合：

1. レビュー指摘事項をTODOリストに追加
2. container-use環境内で修正を実施
3. テスト再実行で問題なしを確認
4. **再度レビューエージェントを呼び出し**（スキップ禁止）

```python
# 修正後の再レビュー
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

#### 7.6 レビュー失敗時のエスカレーション

3回連続でスコア9点未満の場合：

1. Draft PRを作成（`--draft`フラグ）
2. PRの本文に「レビュー未通過」と明記
3. 未解決の指摘事項をPRコメントに記載
4. ユーザーに報告して判断を仰ぐ

### 7.7. ユーザー承認ゲート ⚠️ 必須

> **⚠️ 重要**: PR作成前に必ずユーザーの承認を得ること。自動でPRを作成しない。

品質レビュー通過後（9点以上）、PR作成前にユーザーに確認を求めます。

#### 承認リクエストフォーマット

```markdown
## ✅ 品質レビュー通過 - PR作成承認リクエスト

### Issue情報
- **Issue**: #{issue_id} - {issue_title}
- **ブランチ**: `feature/issue-{issue_id}-{description}`

### レビュー結果
- **スコア**: {score}/10
- **レビュアー**: {reviewer_agent}

### 変更概要
- 新規ファイル: {new_files_count}件
- 変更ファイル: {modified_files_count}件
- 削除ファイル: {deleted_files_count}件

### 主な変更内容
{change_summary}

### テスト結果
- 合計: {total_tests}件
- 成功: {passed_tests}件
- 失敗: {failed_tests}件

---

**PR作成を承認しますか？**
- `続行`: PR作成を続行
- `修正`: 追加修正が必要（指摘箇所をコメントしてください）
- `下書き`: Draft PRとして作成
```

#### 承認フロー

ユーザーに承認リクエストを表示し、`続行`→通常PR、`下書き`→Draft PR、`修正`→修正へ戻る。

#### 承認結果に応じたアクション

| ユーザー回答 | アクション |
|------------|----------|
| `続行` | 通常PRを作成 → Phase 8へ |
| `下書き` | Draft PRを作成（`--draft`フラグ付き） |
| `修正` + フィードバック | 指摘箇所を修正 → Phase 6（Lint & Test）へ戻る |
| タイムアウト（30分） | Draft PRを自動作成、ユーザーに通知 |

#### 承認タイムアウト仕様

| パラメータ | 値 | 説明 |
|----------|-----|------|
| タイムアウト時間 | 30分 | ユーザー応答の待機上限 |
| タイムアウト時の挙動 | Draft PR作成 | 作業成果を保全 |
| 再開方法 | PRページで承認/修正指示 | Draft解除またはコメント |

### 8. コミット & プッシュ (container内で実行)

```python
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command='''
        git add . && \
        git commit -m "feat: {summary}

Closes #{issue_id}

- {change1}
- {change2}" && \
        git push origin feature/issue-{issue_id}-{description}
    '''
)
```

**コミットメッセージ規則**:
- `feat:` - 新機能
- `fix:` - バグ修正
- `refactor:` - リファクタリング
- `test:` - テスト追加
- `docs:` - ドキュメント

### 9. PR作成 (container内で実行)

> **⚠️ 重要**: PRのタイトルと本文は**日本語**で記述してください。

```python
container-use_environment_run_cmd(
    environment_id=env_id,
    environment_source="/path/to/repo",
    command='''
        gh pr create \
          --title "feat: {日本語タイトル}" \
          --body "## 概要
Closes #{issue_id}

{変更の概要を日本語で記述}

## 変更内容
- {変更点1}
- {変更点2}

## テスト結果
{test_log}

## チェックリスト
- [x] TDDで実装
- [x] 品質レビュー通過
- [x] Lintエラーなし
- [x] 型エラーなし" \
          --base main \
          --head feature/issue-{issue_id}-{description}
    '''
)
```

**PRタイトル形式（日本語）**:
| プレフィックス | 用途 | 例 |
|---------------|------|-----|
| `feat:` | 新機能 | `feat: ポモドーロタイマーの基本データ型を追加` |
| `fix:` | バグ修正 | `fix: タイマー停止時のエラーを修正` |
| `refactor:` | リファクタリング | `refactor: 設定管理のコードを整理` |
| `test:` | テスト追加 | `test: IPC通信のユニットテストを追加` |
| `docs:` | ドキュメント | `docs: READMEにインストール手順を追加` |

### 10. CI監視 & 自動マージ ⚠️ 必須

> **⚠️ 重要**: PR作成後、CIの完了を待ち、結果に応じて自動マージまたは修正を行う。

#### 実行者の責任分担

| フェーズ | 実行者 | 理由 |
|---------|--------|------|
| 0-9 (実装→PR作成) | `container-worker` (並列時) / `Sisyphus` (単一時) | container-use環境内での作業 |
| **10 (CI監視→マージ)** | **`Sisyphus` (親エージェント)** | GitHub API操作、環境外での監視 |
| **11 (環境クリーンアップ)** | **`Sisyphus` (親エージェント)** | 環境管理はホスト側で実行 |

> **Note**: セクション10-11はcontainer-use環境**外**で実行します。
> CI監視やPRマージはGitHub APIの呼び出しであり、環境内のファイル操作ではないため`bash`ツールの使用が許容されます。

PR作成後、以下のフローを実行します：

<!-- [DIAGRAM-FOR-HUMANS] CI監視フロー図（AI処理時はスキップ）
PR作成 → CI待機(10分) → 成功:マージ→環境削除 / 失敗:ログ分析→修正→push(3回まで) / 3回超過:エスカレーション
-->

#### 10.1 CI完了待機

```python
def wait_for_ci(pr_number: int, timeout: int = 600) -> CIResult:
    """30秒間隔でgh pr checksをポーリング（最大10分）"""
    # 全SUCCESS → SUCCESS、1つでもFAILURE → FAILURE、タイムアウト → TIMEOUT
    for _ in range(timeout // 30):
        checks = bash(f"gh pr checks {pr_number} --json state,name")
        if all_success(checks): return SUCCESS
        if any_failure(checks): return FAILURE
        wait(30)
    return TIMEOUT

def handle_ci_timeout(pr_number: int, env_id: str):
    """タイムアウト時: pending_checksあり→「CI実行中」、なし→「状態取得エラー」を報告"""
    report_to_user(f"⏱️ CI待機タイムアウト PR #{pr_number}。gh pr checks --watch で手動確認")
```

#### 10.2 CI失敗時の修正フロー

```python
MAX_CI_RETRIES = 3  # CIリトライ上限

def handle_ci_failure(pr_number: int, env_id: str) -> bool:
    """CI失敗 → ログ分析 → container環境で修正 → push → 再待機（最大3回）"""
    for attempt in range(MAX_CI_RETRIES):
        log = bash("gh run view --log-failed")
        fix_in_container(env_id, analyze_failure(log))
        bash("git add . && git commit -m 'fix: CI修正' && git push")
        if wait_for_ci(pr_number) == SUCCESS:
            return True
    return False  # リトライ超過 → escalate_ci_failure()
```

#### 10.3 リトライ上限超過時のエスカレーション

```python
def escalate_ci_failure(pr_number: int, env_id: str):
    """PRをDraft化、失敗ログをコメント、ユーザーに報告"""
    bash(f"gh pr ready {pr_number} --undo")
    bash(f"gh pr comment {pr_number} --body '⚠️ CI修正3回失敗。env_id: {env_id}'")
    report_to_user(f"⚠️ PR #{pr_number} 手動確認が必要")
```

#### 10.4 自動マージ

```python
def auto_merge_pr(pr_number: int, env_id: str) -> bool:
    """gh pr merge --merge --delete-branch。失敗時はhandle_merge_failure()"""
    result = bash(f"gh pr merge {pr_number} --merge --delete-branch")
    return result.exit_code == 0 or handle_merge_failure(pr_number, error=result.stderr)
    # handle_merge_failure: conflict → checkout案内, protected branch → レビュー確認案内
```

#### 10.5 CI監視のメインフロー

```python
def post_pr_workflow(pr_number: int, env_id: str):
    """PR作成後: CI待機 → 成功:マージ&削除 / 失敗:修正(3回) / タイムアウト:報告"""
    ci_result = wait_for_ci(pr_number)
    
    if ci_result == SUCCESS:
        auto_merge_pr(pr_number, env_id) and cleanup_environment(env_id)
    elif ci_result == FAILURE:
        handle_ci_failure(pr_number, env_id) and auto_merge_pr(...) and cleanup_environment(...)
        # 修正失敗時 → escalate_ci_failure() 環境保持
    elif ci_result == TIMEOUT:
        handle_ci_timeout(pr_number, env_id)  # 環境保持
```

### 11. 環境クリーンアップ ⚠️ 必須

> **⚠️ 重要**: PRマージ後、使用したcontainer-use環境を削除する。

```python
def cleanup_environment(env_id: str, pr_number: int) -> bool:
    """container-use delete {env_id} を実行（最大2回リトライ）"""
    for _ in range(3):  # MAX_CLEANUP_RETRIES + 1
        if bash(f"container-use delete {env_id}").exit_code == 0:
            report_to_user(f"✅ PR #{pr_number} マージ済み、環境 {env_id} 削除済み")
            return True
        wait(5)
    report_to_user(f"⚠️ 環境削除失敗。手動: container-use delete {env_id}")
    return False
```

#### クリーンアップのタイミング

| 状況 | 環境の扱い |
|------|----------|
| PRマージ成功 | ✅ 即座に削除 |
| PRクローズ（マージなし） | ✅ 即座に削除 |
| CI修正中（リトライ中） | ❌ 削除しない（同じ環境で作業継続） |
| Draft PR（エスカレーション中） | ❌ 削除しない（手動修正用に保持） |
| PRレビュー修正待ち | ❌ 削除しない（修正用に保持） |

### 12. 親Issue自動クローズ ⚠️ 必須

> **⚠️ 重要**: 全SubtaskのPRがマージされたら、親Issueを自動でクローズする。

#### 12.1 Subtask完了チェック

```python
def check_all_subtasks_complete(parent_issue_id: int) -> bool:
    """親Issueに紐づく全Subtaskが完了したかチェック"""
    
    # detect_subtasks() を再利用（重複ロジック回避）
    # ※ detect_subtasks() は「引数」セクションで定義済み
    subtask_ids = detect_subtasks(parent_issue_id)
    
    if not subtask_ids:
        # Subtaskがない場合は親Issue自体の完了をチェック
        return True
    
    # 各SubtaskのステータスとPRマージ状況を確認
    for subtask_id in subtask_ids:
        result = bash(f"gh issue view {subtask_id} --json state")
        if result.exit_code != 0:
            continue
        
        issue_data = json.loads(result.stdout)
        if issue_data.get("state") != "CLOSED":
            return False
        
        # 関連PRがマージされているか確認
        pr_result = bash(f"gh pr list --search 'closes #{subtask_id}' --state merged --json number")
        if pr_result.exit_code != 0 or not json.loads(pr_result.stdout):
            return False
    
    return True
```

> **Note**: `detect_subtasks()` は「引数」セクションで定義されている共通関数。
> Subtask検出ロジックの重複を避けるため、必ずこの関数を再利用すること。

#### 12.2 親Issueクローズ処理

```python
def close_parent_issue(parent_issue_id: int, subtask_results: list[dict]):
    """全Subtask完了後、親Issueをクローズ"""
    
    # サマリーコメントを作成
    summary = f"""
## ✅ 全Subtask完了

| Subtask | PR | ステータス |
|---------|-----|----------|
"""
    for r in subtask_results:
        summary += f"| #{r['subtask_id']} | PR #{r['pr_number']} | ✅ Merged |\n"
    
    summary += f"""
---
🤖 全{len(subtask_results)}件のSubtaskが正常にマージされました。
このIssueを自動クローズします。
"""
    
    # コメント追加
    bash(f'''
        gh issue comment {parent_issue_id} --body "{summary}"
    ''')
    
    # 親Issueをクローズ
    bash(f"gh issue close {parent_issue_id} --reason completed")
    
    report_to_user(f"✅ 親Issue #{parent_issue_id} を自動クローズしました")
```

#### 12.3 部分完了時の処理

| 状況 | アクション |
|------|----------|
| 全Subtask成功 | 親Issueを自動クローズ |
| 一部Subtask失敗 | 親Issueは開いたまま、失敗Subtaskを報告 |
| 全Subtask失敗 | 親Issueにエラーサマリーをコメント |

```python
def handle_partial_completion(parent_issue_id: int, results: list[dict]):
    """部分完了時の処理"""
    
    succeeded = [r for r in results if r['status'] == 'merged']
    failed = [r for r in results if r['status'] != 'merged']
    
    if not failed:
        # 全成功 → 親Issueクローズ
        close_parent_issue(parent_issue_id, succeeded)
    else:
        # 一部失敗 → 報告のみ
        comment = f"""
## ⚠️ 一部Subtaskが未完了

### ✅ 成功 ({len(succeeded)}件)
{format_subtask_list(succeeded)}

### ❌ 失敗/未完了 ({len(failed)}件)
{format_subtask_list(failed)}

---
失敗したSubtaskを修正後、再度 `/implement-issues {' '.join(str(f['subtask_id']) for f in failed)}` を実行してください。
"""
        bash(f"gh issue comment {parent_issue_id} --body '{comment}'")
```

### 13. 並列処理時のCI監視

> **⚡ トークン効率**: CI監視はエージェント起動せず、bash直接実行で行う。

複数PRのCI監視は**bashツールで直接実行**（エージェント起動不要、~2,000トークン/PR削減）。

```python
def post_pr_workflow_parallel(pr_results: list[dict]):
    """各PRに対してmonitor_ci_direct()を実行 → 成功:マージ&削除 / 失敗:環境保持"""
    for r in pr_results:
        status = monitor_ci_direct(r['pr_number'], r['env_id'])  # bash直接
        # 成功: gh pr merge + container-use delete
        # 失敗/タイムアウト: 環境保持、report_to_user()
```

## 技術スタック別設定

詳細は [container-use環境構築ガイド](../skill/container-use-guide.md) を参照。

| スタック | base_image | setup_commands |
|---------|------------|----------------|
| Node.js/TypeScript | `node:20-slim` | `npm ci` |
| Python | `python:3.11-slim` | `pip install -r requirements.txt` |
| Go | `golang:1.21` | `go mod download` |
| Rust | `rust:1.85-slim` | `cargo fetch` |

## エラーハンドリング

### GitHub API エラー

| 状況 | 対応 |
|------|------|
| Issue不存在（404） | エラーメッセージを表示し、Issue番号の確認を依頼 |
| レート制限（403） | 1分待機後にリトライ（最大3回） |
| ネットワークエラー | 30秒待機後にリトライ（最大3回） |
| 認証エラー（401） | `gh auth login` の実行を案内 |

```python
def safe_gh_api_call(command: str, max_retries: int = 3) -> tuple[bool, str]:
    """GitHub API呼び出しのラッパー（リトライ付き）"""
    for attempt in range(max_retries):
        result = bash(command)
        
        if result.exit_code == 0:
            return (True, result.stdout)
        
        error = result.stderr.lower()
        
        if "404" in error or "not found" in error:
            return (False, f"Issue/PRが見つかりません: {command}")
        
        if "401" in error or "authentication" in error:
            return (False, "認証エラー: `gh auth login` を実行してください")
        
        if "403" in error or "rate limit" in error:
            wait(60)  # レート制限: 1分待機
            continue
        
        # その他のエラー: リトライ
        wait(30)
    
    return (False, f"APIエラー（{max_retries}回リトライ後）: {command}")
```

### 単一Issue処理時

| 状況 | 対応 |
|------|------|
| Issue不存在 | エラー報告して終了 |
| Subtask検出失敗 | ユーザーに確認（続行 or 中断） |
| 3回連続レビュー失敗 | Draft PRを作成して終了 |
| 設計不備 | `/request-design-fix` を実行 |
| 環境構築失敗 | `container-use_environment_config` で設定見直し |
| サービス接続失敗 | ポート・環境変数を確認 |
| ブランチ作成失敗 | 既存ブランチの有無を確認、競合解消 |

### 並列処理時

| 状況 | 対応 |
|------|------|
| 1つのIssueが失敗 | 他のIssueは継続、失敗分のみ報告 |
| 全Issueが失敗 | 各失敗理由を収集して報告 |
| container-worker タイムアウト | タイムアウトしたIssueをリストアップ |
| 依存関係エラー | 依存元Issueを先に処理するよう順序変更 |
| 循環依存検出 | エラー報告し、手動での依存解消を依頼 |
| ブランチ競合 | 競合したIssueのみ報告、他は継続 |

### Subtask検出時のエラー

| 状況 | 対応 |
|------|------|
| 親Issue不存在 | エラー報告して終了 |
| Subtask 0件検出 | 粒度チェックへ移行（正常フロー） |
| 一部Subtaskがクローズ済み | 未完了分のみ実装対象に |
| Subtask循環参照 | エラー報告、手動確認を依頼 |

### 並列処理の結果報告フォーマット

```markdown
## 実装結果サマリー

| Issue | ステータス | PR | レビュースコア |
|-------|----------|-----|--------------|
| #9 | ✅ 成功 | PR #25 | 10/10 |
| #10 | ✅ 成功 | PR #26 | 9/10 |
| #11 | ❌ 失敗 | - | - |

### 失敗詳細

#### Issue #11
- 失敗理由: レビュースコア未達（7/10）
- 指摘事項: ...
- 推奨アクション: 指摘事項を修正して再実行
```

## Sisyphusへの指示（必読）

> **このセクションはSisyphus専用の実行指示です。上記ルールの要約版。**

### 🔄 実装フロー

```
1. Issue受領
     ↓
2. 【単一Issue指定時】Subtask自動検出 ★重要★
     ├─ Subtaskあり → 並列実装へ（Step 4へ）
     └─ Subtaskなし → 粒度チェックへ（Step 3へ）
     ↓
3. 粒度チェック（200行以下か?）
     ├─ No（大きい）→ `/decompose-issue` を実行してから再度呼び出し
     └─ Yes（適切）→ 実装開始
     ↓
4. 各Issueを並列実行（container-worker）
     ↓
5. CI監視 → マージ（各Issueごと）
     ↓
6. 全Issue完了 → 親Issue自動クローズ（該当する場合）
```

### ⚡ Subtask自動検出（単一Issue指定時は必須）

```python
# /implement-issues 8 のように単一Issue指定された場合
def handle_single_issue(issue_id: int):
    """単一Issue指定時のSubtask検出フロー"""
    
    # Step 1: Subtask検出
    subtasks = detect_subtasks(issue_id)
    
    if subtasks:
        # Step 2a: Subtaskあり → 並列実装
        report_to_user(f"""
📋 **親Issue #{issue_id} から {len(subtasks)}件のSubtaskを検出しました**

Subtask: {', '.join(f'#{s}' for s in subtasks)}

これらを並列実装します。
""")
        # background_task で並列起動
        for subtask_id in subtasks:
            background_task(
                agent="container-worker",
                description=f"Issue #{subtask_id} 実装",
                prompt=build_worker_prompt(subtask_id)
            )
    else:
        # Step 2b: Subtaskなし → 粒度チェック
        if estimate_code_lines(issue_id) > 200:
            report_to_user(f"""
⚠️ Issue #{issue_id} は200行を超える見込みで、Subtaskも検出されませんでした。

先に分解してください:
```bash
/decompose-issue {issue_id}
```
""")
            return
        
        # 単体実装（container-workerを1つ起動）
        implement_single_issue(issue_id)

def implement_single_issue(issue_id: int):
    """
    単体Issue実装（Subtaskなし、200行以下の場合）
    
    ⚠️ 重要: 単体でも container-worker を使用する（一貫性のため）
    """
    # container-worker を1つ起動
    background_task(
        agent="container-worker",
        description=f"Issue #{issue_id} 単体実装",
        prompt=build_worker_prompt(issue_id)
    )
    
    # 結果を待機
    result = background_output(task_id="...")
    
    # CI監視 → マージ（Sisyphusが実行）
    if result.get("pr_number"):
        post_pr_workflow(result["pr_number"], result["env_id"])
```

> **Note**: 単体実装でも `container-worker` を使用する理由:
> - container-use環境ルールの一貫性を保つ
> - Sisyphusがホスト環境でファイル編集しない
> - CI/マージ処理はSisyphusが担当（Phase 10-11）

### 粒度判定（実装開始前に必須）

| 推定コード量 | 対応 |
|-------------|------|
| **200行以下** | → そのまま実装 |
| **200行超** | → **`/decompose-issue` で分割してから再実行** |

```python
# 粒度チェックの例
if estimate_code_lines(issue) > 200:
    report_to_user(f"""
⚠️ Issue #{issue_id} は200行を超える見込みです。

先に分解してください:
```bash
/decompose-issue {issue_id}
```
""")
    return  # 実装を開始しない
```

### 実装フロー（分岐条件）

| 状況 | 処理方法 | 実行者 |
|------|---------|--------|
| **適切な粒度（200行以下）** | 直接実装 | container-worker |
| **大きなIssue（200行超）** | → `/decompose-issue` で分割 | Sisyphus |
| **複数Issue** | 並列実装 | container-worker × N |

### Phase別の責任分担

| Phase | 実行者 | 内容 |
|-------|--------|------|
| **0-9. 実装→PR** | container-worker | TDD、レビュー、PR作成 |
| **10-11. CI→マージ** | Sisyphus | CI監視、マージ、環境削除 |
| **12. 親Issueクローズ** | Sisyphus | 全Subtask完了確認、親Issue自動クローズ |

### ⛔ 必須チェックリスト

```
□ 【単一Issue指定時】Subtask検出を実行したか? ★最優先★
□ Issue粒度チェック（200行以下か?）
□ 大きい場合は `/decompose-issue` を案内したか?
□ background_task を使用しているか?（⛔ task 禁止）
□ 各Issueに独立した container-worker を起動するか?
□ 全Subtask完了後、親Issueをクローズしたか?
```

### ツール使用ルール

| 操作 | 使用ツール | 備考 |
|------|-----------|------|
| container-worker起動 | `background_task` | ⛔ `task` 禁止（MCPツール継承されない） |
| 品質レビュー起動 | `task` | ✅ OK（レビューエージェントはMCP不要） |
| ファイル操作 | `container-use_environment_file_*` | ⛔ `edit`/`write` 禁止 |
| コマンド実行 | `container-use_environment_run_cmd` | ⛔ `bash` でのテスト/ビルド禁止 |
| CI監視・マージ | `bash` (gh コマンド) | ✅ OK（環境外のGitHub API操作） |
| 親Issueクローズ | `bash` (gh issue close) | 全Subtask完了後 |

### ⛔ よくある間違い

| ❌ 間違い | ✅ 正しい方法 |
|----------|-------------|
| **単一Issue指定時にSubtask検出をスキップ** | **必ず `detect_subtasks()` を実行** |
| 親IssueをそのままSubtaskなしで実装開始 | まずSubtask検出 → なければ粒度チェック |
| 大きなIssueをそのまま実装 | `/decompose-issue` で分割してから実装 |
| `task(subagent_type="container-worker", ...)` | `background_task(agent="container-worker", ...)` |
| 結果を待たずに次へ進む | `background_output` で全結果を収集してから報告 |
| 全Subtask完了後、親Issueを放置 | 必ず自動クローズ処理を実行 |

### 完了報告フォーマット

```markdown
## 📋 実装完了サマリー

### 親Issue
- **#{parent_id}**: {parent_title} → ✅ Closed

### Subtask結果

| Subtask | PR | CI | マージ | 行数 |
|---------|-----|-----|-------|------|
| #{s1} | PR #{p1} | ✅ | ✅ | 80行 |
| #{s2} | PR #{p2} | ✅ | ✅ | 120行 |
| #{s3} | PR #{p3} | ✅ | ✅ | 150行 |

### 統計
- 総Subtask数: 3
- 成功: 3
- 失敗: 0
- 合計コード行数: 350行

### 環境クリーンアップ
- ✅ 全環境削除済み
```

## 参考

- [container-use環境構築ガイド](../skill/container-use-guide.md)
- [申し送り処理ガイド](../skill/handover-process.md)
- [コード品質ルール](../skill/code-quality-rules.md)
