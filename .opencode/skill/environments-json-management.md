# environments.json 管理

> **参照元**: implement-issues.md から分離された環境管理ロジック

---

## 概要

環境IDは `.opencode/environments.json` で追跡する。

詳細は [container-use環境構築ガイド](./container-use-guide.md) を参照。

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
