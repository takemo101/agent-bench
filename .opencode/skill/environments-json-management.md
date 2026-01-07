# environments.json 管理

> **Single Source of Truth**: container-use環境の状態管理

---

## 概要

**ALL container-use operations MUST update `.opencode/environments.json`** to track Issue/PR/Environment relationships.

---

## ファイル構造

### パス

```
.opencode/environments.json
```

### 初期化

ファイルが存在しない場合、以下の構造で作成：

```json
{
  "$schema": "./environments.schema.json",
  "environments": []
}
```

### データ構造

```json
{
  "env_id": "abc-123-def",
  "branch": "feature/issue-42-user-auth",
  "issue_number": 42,
  "pr_number": null,
  "title": "User authentication feature",
  "status": "active",
  "created_at": "2026-01-03T10:00:00Z",
  "last_used_at": "2026-01-03T15:30:00Z"
}
```

### ステータス値

| ステータス | 説明 |
|-----------|------|
| `active` | 作業中 |
| `pr_created` | PR作成済み |
| `merged` | PRマージ済み |
| `abandoned` | 放棄（PRクローズ等） |

---

## 必須更新ポイント (NON-NEGOTIABLE)

| トリガー | アクション | 更新フィールド |
|---------|----------|---------------|
| `environment_create` 成功 | **ADD** | `env_id`, `branch`, `issue_number`, `title`, `status: "active"`, `created_at`, `last_used_at` |
| `environment_open` 成功 | **UPDATE** | `last_used_at` |
| `gh pr create` 成功 | **UPDATE** | `pr_number`, `status: "pr_created"`, `last_used_at` |
| PR merged | **UPDATE** | `status: "merged"`, `last_used_at` |
| PR closed (マージなし) | **UPDATE** | `status: "abandoned"`, `last_used_at` |
| 環境削除 | **REMOVE** | エントリ全体を削除 |

---

## API

```python
import json
from pathlib import Path
from datetime import datetime

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
        "issue_number": issue_id,  # JSON field name: issue_number
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
        if env["issue_number"] == issue_id and env["status"] in ["active", "pr_created"]:
            return env
    return None
```

---

## 更新タイミング

| イベント | 関数 | 更新内容 |
|---------|------|---------|
| 環境作成時 | `register_environment()` | 新規登録 |
| PR作成時 | `update_environment_pr()` | PR番号記録 |
| PRマージ後 | `mark_environment_merged()` | ステータス更新 |
| 環境削除時 | `remove_environment()` | レコード削除 |
| PR修正時 | `find_environment_by_issue()` | 既存環境を再利用 |

---

## セッション復旧

作業再開時、**environments.json を最優先で参照**：

```bash
# 1. Read .opencode/environments.json
# 2. Find entry matching the Issue number or PR number
# 3. Use the stored env_id to reopen environment
```

### 復旧判定マトリクス

| Entry Status | PR State | Action |
|--------------|----------|--------|
| `active` | No PR | 作業継続、`env_id` で環境再開 |
| `pr_created` | PR open | 修正用に `env_id` で環境再開 |
| `pr_created` | PR merged | `status: "merged"` に更新、環境削除 |
| `merged` | N/A | クリーンアップ候補 |
| `abandoned` | N/A | 環境とエントリを即削除 |

---

## クリーンアップポリシー

| 条件 | アクション |
|------|----------|
| `status: "merged"` から 7日以上 | 環境削除 + エントリ削除 |
| `status: "abandoned"` | 即時削除 |
| `last_used_at` から 30日以上 | レビューして削除検討 |

---

## ハードブロック (違反禁止)

| 違反 | 結果 |
|------|------|
| 環境作成時に environments.json 未登録 | **FORBIDDEN** - 復旧不可 |
| PR作成時に environments.json 未更新 | **FORBIDDEN** - 追跡不可 |
| 環境削除時に environments.json 未更新 | **FORBIDDEN** - stale data |

---

## 並列実行時のルール

| Actor | 責任 |
|-------|------|
| `container-worker` | `env_id` を最終レスポンスで返却。environments.json は**更新しない** |
| Main agent (Sisyphus) | 全worker完了後に environments.json を一括更新 |

> **理由**: 並列書き込みによる競合を回避

---

## 変数命名規則

| Context | Variable Name | Rationale |
|---------|---------------|-----------|
| **environments.json** (data) | `issue_number` | JSON field name (GitHub API準拠) |
| **Code/Pseudocode** | `issue_id` | 一般的な変数名規約 |

```python
# Writing
entry = {"issue_number": issue_id, ...}

# Reading
issue_id = entry["issue_number"]
```
