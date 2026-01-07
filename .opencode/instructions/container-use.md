# Container-Use Agent Rules

## Core Rules (MANDATORY)

ALWAYS use ONLY Environments for ANY and ALL file, code, or shell operations—NO EXCEPTIONS—even for simple or generic requests.

**Note**: See "When to Use Container-Use" section for the only permitted exception (`.opencode/` workflow documentation).

DO NOT install or use the git cli with the environment_run_cmd tool. All environment tools will handle git operations for you. Changing ".git" yourself will compromise the integrity of your environment.

You MUST inform the user how to view your work using `container-use log <env_id>` AND `container-use checkout <env_id>`. Failure to do this will make your work inaccessible to others.

### Environment Integrity Protocol

If local build/test commands fail due to environment issues (e.g., wrong rustc version):
1. **STOP**. Do NOT push to CI hoping it works there.
2. **FIX** the environment (or switch to a Container).
3. **VERIFY** locally.
4. Only then, **PUSH**.

**Pushing broken code to CI to test it is strictly FORBIDDEN.**

---

## environments.json Management (MANDATORY)

**ALL container-use operations MUST update `.opencode/environments.json`** to track Issue/PR/Environment relationships.

### File Location & Initialization

```
.opencode/environments.json
```

If the file does not exist, create it with the initial structure:

```json
{
  "$schema": "./environments.schema.json",
  "environments": []
}
```

### Data Structure

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

**Valid status values**: `active`, `pr_created`, `merged`, `abandoned`

### MANDATORY Update Points (NON-NEGOTIABLE)

| Trigger | Required Action | Fields to Update |
|---------|----------------|------------------|
| `environment_create` success | **ADD** new entry | `env_id`, `branch`, `issue_number`, `title`, `status: "active"`, `created_at`, `last_used_at` |
| `environment_open` success | **UPDATE** existing entry | `last_used_at` |
| `gh pr create` success | **UPDATE** existing entry | `pr_number`, `status: "pr_created"`, `last_used_at` |
| PR merged | **UPDATE** existing entry | `status: "merged"`, `last_used_at` |
| PR closed (without merge) | **UPDATE** existing entry | `status: "abandoned"`, `last_used_at` |
| Environment deleted | **REMOVE** entry | Delete entire entry from array |

### Implementation (Use Read/Edit Tools)

**After `environment_create`:**
```bash
# 1. Read current file (or create if not exists)
# 2. Add new entry to environments array
# 3. Write updated file
```

Example entry to add:
```json
{
  "env_id": "<returned_env_id>",
  "branch": "feature/issue-<N>-<description>",
  "issue_number": <N>,
  "pr_number": null,
  "title": "<environment_title>",
  "status": "active",
  "created_at": "<ISO8601_timestamp>",
  "last_used_at": "<ISO8601_timestamp>"
}
```

**After `gh pr create`:**
```bash
# 1. Read environments.json
# 2. Find entry by env_id
# 3. Update pr_number and status
# 4. Write updated file
```

### Session Recovery (MANDATORY)

When resuming work, **ALWAYS check environments.json FIRST**:

```bash
# 1. Read .opencode/environments.json
# 2. Find entry matching the Issue number or PR number
# 3. Use the stored env_id to reopen environment
```

**Decision Matrix based on environments.json:**

| Entry Status | PR State | Action |
|--------------|----------|--------|
| `active` | No PR | Continue work, reopen with stored `env_id` |
| `pr_created` | PR open | Reopen with stored `env_id` for fixes |
| `pr_created` | PR merged | Update status to `merged`, delete env |
| `merged` | N/A | No action needed (cleanup candidate) |
| `abandoned` | N/A | Delete environment and entry |

### Cleanup Policy

| Condition | Action |
|-----------|--------|
| Status `merged` for 7+ days | Delete environment + remove entry |
| Status `abandoned` | Delete immediately |
| `last_used_at` > 30 days | Review and consider deletion |

### Hard Blocks

| Violation | Consequence |
|-----------|-------------|
| Creating environment without adding to environments.json | **FORBIDDEN** - breaks recovery |
| Creating PR without updating environments.json | **FORBIDDEN** - loses tracking |
| Deleting environment without removing from environments.json | **FORBIDDEN** - stale data |

---

## When to Use Container-Use

| Use Container-Use | Do NOT Use |
|-------------------|------------|
| Issue implementation (code changes) | Research / investigation only |
| New feature development | Documentation review |
| Bug fixes | Design discussions / reviews |
| Refactoring | Reading existing code |

**Decision criteria**: Will you modify files? → YES → Container-Use

**Exception**: `.opencode/` workflow documentation (instructions, skills, agents) may be edited directly on host when:
- Changes are documentation-only (no code impact)
- Quick iteration is needed for workflow improvements
- Docker is unavailable and user approves direct editing

---

## Required Parameters

All `container-use_*` tools require:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `environment_source` | Absolute path to project git repository | `/Users/user/projects/my-app` |
| `environment_id` | Environment UUID (obtained after create) | `env-abc123...` |

**Notes**:
- `environment_source`: Use current working directory. If unknown, ask the user
- `environment_id`: NOT required for `environment_create` (returned as create result). Required for all other tools

---

## Execution Options

| Method | Use Case | Characteristics |
|--------|----------|-----------------|
| **Direct tool execution** | Single Issue, sequential work | Call `container-use_*` tools directly. Simple and controllable |
| **task delegation (parallel)** | Multiple Issues simultaneously | Each worker operates in independent environment |

### Single Issue → Direct Tool Execution

```
1. Create environment with environment_create
2. Work directly with environment_file_* / environment_run_cmd
3. Present Environment Info to user upon completion
```

### Multiple Issues Parallel → task Delegation

When implementing different Issues simultaneously, delegate to `container-worker` via `task`:

```python
# Delegate Issue implementations in parallel
task(subagent_type="container-worker", prompt="Issue #1: User authentication...")
task(subagent_type="container-worker", prompt="Issue #2: Notification system...")
task(subagent_type="container-worker", prompt="Issue #3: Dashboard...")
# Each worker creates and manages independent environment
```

**Important**:
- Record the Environment ID returned by each worker
- Upon completion, aggregate and present Environment Info for all environments to user

### environments.json in Parallel Execution

**Concurrency Rule**: Only the **main agent (Sisyphus)** updates `environments.json`.

| Actor | Responsibility |
|-------|---------------|
| `container-worker` | Returns `env_id` in final response. Does NOT update environments.json |
| Main agent | Collects all `env_id` values and updates environments.json after all workers complete |

**Workflow**:
1. Main agent creates todo list for parallel issues
2. Delegates to `container-worker` agents (they work independently)
3. Each worker returns: `env_id`, `branch`, `issue_number`, `pr_number` (if created)
4. Main agent updates `environments.json` with all entries at once
5. This avoids race conditions and file conflicts

### container-worker Delegation Prompt Structure

When delegating via task, include the following information:

```markdown
1. ISSUE: Issue number and summary
2. REPOSITORY: Path for environment_source
3. GOAL: Specific objectives to achieve
4. SCOPE: Target files/directories to modify
5. CONSTRAINTS: Actions that are forbidden
6. VERIFICATION: How to verify completion (tests, build, etc.)
```

---

## Environment Lifecycle Management

### Environment Creation
- ALWAYS create a new environment at the start of a new task/issue
- Record the `env_id` immediately after creation
- Use descriptive environment names matching the task (e.g., `feature-issue-8-sound`)

### Environment Persistence
- NEVER abandon an environment due to errors
- If an operation fails, diagnose and retry within the SAME environment
- Use `environment_open` to reconnect to existing environments

### Environment Reuse Rules

| Situation | Action |
|-----------|--------|
| Same issue, continuing work | Reuse existing environment via `environment_open` |
| PR review feedback/fixes | Reuse the SAME environment (do NOT create new) |
| New issue/feature | Create NEW environment |
| Fix branch for different issue | Create NEW environment |

---

## Crash Recovery Protocol

When encountering errors or crashes:

1. **DO NOT** fall back to direct host file operations
2. **DO NOT** abandon the container-use workflow
3. **INSTEAD**, follow this recovery flow:
   ```
   a. Check environment status with `environment_list`
   b. Reopen the environment with `environment_open(env_id)`
   c. Verify file state with `environment_file_list`
   d. Continue work within the environment
   ```

4. **If environment is corrupted:**
   ```
   a. Update environments.json: mark old entry status as "abandoned"
   b. Create a NEW environment with the same branch
   c. Add new entry to environments.json with new env_id
   d. The git state will be preserved from the remote
   e. Continue work in the new environment
   ```

---

## Error Handling Escalation

1. **First failure**: Retry the operation
2. **Second failure**: Check environment state, diagnose issue
3. **Third failure**: Create checkpoint, attempt alternative approach
4. **Persistent failure**: Report to user WITH environment ID for manual recovery

NEVER silently switch to non-container-use operations.

---

## Docker Resource Failures (Fallback Protocol)

When Docker itself fails (disk space, daemon issues, resource exhaustion):

### Diagnosis Commands

```bash
# Check Docker disk usage
docker system df

# Check available disk space
df -h

# Check Docker daemon status
docker info
```

### Fallback Decision Tree

| Condition | Action |
|-----------|--------|
| Disk space < 10GB | Run `docker system prune -af` and retry |
| Docker daemon not running | Start Docker Desktop, wait 30s, retry |
| After prune still failing | **User decision required** |

### User Escalation (MANDATORY)

When container-use cannot function, you MUST:

1. **Report the failure clearly**:
   ```
   ⚠️ Container-use is unavailable due to: [specific error]
   
   Attempted recovery:
   - [action 1]: [result]
   - [action 2]: [result]
   ```

2. **Present options**:
   ```
   Options:
   A) Wait for Docker recovery (manual intervention needed)
   B) Proceed with direct host operations (breaks isolation)
   C) Abort and resume later
   
   Which would you prefer?
   ```

3. **If user chooses direct host operations**:
   - Document clearly in commit message: `[non-containerized]`
   - Add warning comment at top of changed files:
     ```
     // ⚠️ WARNING: Modified outside container-use (Docker unavailable)
     // Verify in container environment before merging. Ref: Issue #XXX
     ```
   - Create follow-up issue to verify changes in container

**CRITICAL**: Never silently fall back. Always get explicit user approval.

---

## Session Recovery Protocol

When resuming work from a previous session (e.g., after crash, timeout, or interruption):

### Mandatory State Verification (BEFORE any action)

**Step 1: Check environments.json FIRST (MANDATORY)**

```bash
# Read environments.json to find existing environment for the Issue/PR
cat .opencode/environments.json
```

Look for entries matching:
- `issue_number` for the Issue you're working on
- `pr_number` if a PR was already created
- `status: "active"` or `status: "pr_created"` (usable environments)

**Step 2: If environment found in environments.json**

```bash
# Use the env_id from environments.json to reopen
container-use_environment_open(environment_id="<env_id from json>")
```

**Step 3: If NO environment found, verify other state**

```bash
# 1. Check git state
git status
git log --oneline -3

# 2. Check PR state (if PR was being created)
gh pr list --head <branch-name>
gh pr view <pr-number>  # if PR exists

# 3. Check Issue state
gh issue view <issue-number>

# 4. Check environment state via tool
container-use_environment_list
```

**Note**: `container-use_environment_list` is a tool call, not a bash command. Use the agent tool to check environment state.

### Recovery Decision Matrix

| Git State | PR State | Issue State | Action |
|-----------|----------|-------------|--------|
| Changes uncommitted | N/A | OPEN | Continue implementation |
| Changes committed, not pushed | No PR | OPEN | Push and create PR |
| Changes pushed | No PR | OPEN | Create PR |
| Changes pushed | PR exists (open), CI passing | OPEN | Merge PR |
| Changes pushed | PR exists (open), CI failing | OPEN | Fix issues, push, wait for CI |
| Changes pushed | PR closed (not merged) | OPEN | Review feedback, fix, create new PR |
| Changes pushed | PR merged | OPEN | Verify Issue auto-closed, close if needed |
| Changes pushed | PR merged | CLOSED | **DONE** - verify and report |
| Branch deleted on remote | N/A | OPEN | Re-push from local, or restart |
| N/A | N/A | CLOSED (by others) | Verify completion, report status to user |

**Note on Worktree Conflicts**: If `gh pr merge --delete-branch` fails with worktree error, merge without `--delete-branch` flag. Delete branch manually later if needed.

### Continuation Prompt Best Practices

When creating continuation prompts for future sessions:

~~~markdown
## Session Context
- Branch: <branch-name>
- Last commit: <commit-hash>
- Environment ID: <env-id> (if using container-use)

## Completed Steps
- [x] Step 1 (evidence: commit abc123)
- [x] Step 2 (evidence: PR #45)

## Remaining Steps
- [ ] Step 3: <specific action>
- [ ] Step 4: <specific action>

## Verification Commands (run BEFORE resuming)
    git status
    gh pr view <pr-number>
    gh issue view <issue-number>

## CRITICAL: Do NOT assume previous state. Always verify.
~~~

**Anti-pattern**: Blindly executing continuation prompts without state verification.

### Session Summary Auto-Save (NEW)

セッション終了時に Supermemory へ自動保存し、次回セッションでの復旧を確実にします。

#### 保存タイミング

| イベント | 保存内容 | 保存先 |
|---------|---------|--------|
| **Issue 実装開始** | Issue番号、ブランチ名、env_id | environments.json + Supermemory |
| **PR 作成完了** | PR番号、レビュースコア、CI状態 | environments.json + Supermemory |
| **セッション中断** | 現在の作業状態、TODO残項目 | Supermemory (Session Summary) |
| **エラー発生** | エラー内容、復旧手順 | Supermemory |

#### 自動保存フォーマット

```markdown
[Session Summary]

## Summary: {作業内容の要約}

### What We Did
1. {完了した作業1}
2. {完了した作業2}

### Current State
- **Issue**: #{issue_id}
- **Branch**: {branch_name}
- **Environment ID**: {env_id}
- **PR**: #{pr_number} (if created)
- **CI Status**: {passing/failing/pending}

### Files Modified
- {file1}
- {file2}

### Remaining Tasks
- [ ] {残タスク1}
- [ ] {残タスク2}

### Recovery Commands
```bash
# 1. environments.json から env_id を取得
cat .opencode/environments.json | jq '.environments[] | select(.issue_number == {issue_id})'

# 2. 環境を再開
container-use_environment_open(environment_id="{env_id}")

# 3. 状態を確認
gh issue view {issue_id}
gh pr view {pr_number}  # if PR exists
```
```

#### 復旧時の自動検索

次回セッション開始時、関連するメモリを自動検索：

```python
def auto_recover_session(issue_id: int) -> SessionState | None:
    """前回セッションの状態を自動復旧"""
    
    # 1. environments.json をチェック（最優先）
    env_entry = find_environment_by_issue(issue_id)
    if env_entry:
        return SessionState(
            env_id=env_entry["env_id"],
            branch=env_entry["branch"],
            pr_number=env_entry.get("pr_number"),
            status=env_entry["status"]
        )
    
    # 2. Supermemory から検索（フォールバック）
    memory_result = supermemory(
        mode="search",
        query=f"Issue #{issue_id} Session Summary",
        scope="project"
    )
    
    if memory_result.get("results"):
        return parse_session_summary(memory_result["results"][0])
    
    return None  # 前回セッションなし
```

---

## Forbidden Actions (HARD BLOCKS)

| Action | Why It's Forbidden |
|--------|-------------------|
| Direct file read/write on host | Bypasses container isolation |
| Using `bash` for file operations | Must use environment_* tools |
| Abandoning environment on error | Loses work and context |
| Creating environment without recording env_id | Cannot recover later |
| Using git CLI in environment_run_cmd | Corrupts environment git state |

---

## Completion Criteria

Work is complete when ALL conditions are met:

- [ ] Implementation complete (all files edited)
- [ ] Build passes (verify with `environment_run_cmd`, if applicable)
- [ ] Tests pass (if applicable)
- [ ] Environment Info presented (format below)
- [ ] PR created (using PR Description Template below)
- [ ] **environments.json updated**: `pr_number` set, `status: "pr_created"`
- [ ] **CI passed** (MUST wait: `gh pr checks <pr-number> --watch`)
- [ ] PR merged (only AFTER CI passes)
- [ ] Issue closed (automatic if `Closes #XX` used in PR)
- [ ] **environments.json updated**: `status: "merged"` or entry removed
- [ ] **Environment deleted**: `container-use delete <env_id>` (after PR merge)
- [ ] **Remote branch deleted**: `git push origin --delete <branch-name>` (after PR merge)

### PR Merge Flow (MANDATORY)

```bash
# 1. Create PR (with "Closes #XX" in body for auto-close)
gh pr create --title "..." --body "..."

# 2. Update environments.json (MANDATORY)
# - Set pr_number to the created PR number
# - Set status to "pr_created"
# - Update last_used_at

# 3. Wait for CI to complete (NEVER skip this step)
gh pr checks <pr-number> --watch

# 4. Merge only after CI passes
gh pr merge <pr-number> --merge

# 5. Verify issue auto-closed (if "Closes #XX" was used)
gh issue view <issue-number>  # Should show "CLOSED"

# 6. Clean up
git push origin --delete <branch-name>  # Delete remote branch
container-use delete <env_id>           # Delete environment

# 7. Update environments.json (MANDATORY)
# - Either set status to "merged" OR remove entry entirely
```

**Merge Strategy**:
- `--merge`: Default. Preserves commit history.
- `--squash`: Use for feature branches with many WIP commits.
- `--rebase`: Use when linear history is required.

**Worktree Conflict**: If adding `--delete-branch` option and it fails due to worktree, merge without it. Delete branch manually later if needed.

**HARD BLOCK**: Never merge a PR without confirming CI success.

### PR Description Template (MANDATORY)

Use the following format when creating PRs with `gh pr create`:

```bash
gh pr create --title "the pr title" --body "$(cat <<'EOF'
## Summary
<1-3 bullet points summarizing changes>

## Related Issues
Closes #XX

## Changes
- <specific change 1>
- <specific change 2>

## Testing
- [ ] `cargo test` / `npm test` passed
- [ ] `cargo clippy` / `npm run lint` passed
- [ ] Manual verification (if applicable)

## Design Document Alignment
- [ ] Implementation matches design document
- [ ] OR: Deviations documented below

### Deviations from Design (if any)
<List any intentional differences from design docs with reasoning>
EOF
)"
```

**Key Points**:
- `Closes #XX` automatically closes the related Issue when PR is merged
- Multiple issues: `Closes #12, Closes #13`
- Design alignment section ensures traceability

### Required Outputs

After ANY container-use session, ALWAYS provide:

```
## Environment Info
- Environment ID: `<env_id>`
- View logs: `container-use log <env_id>`
- Checkout code: `container-use checkout <env_id>`
```

---

## Quick Reference

### Common Operations

| Task | Tool to Use |
|------|-------------|
| Create new environment | `environment_create` |
| Reopen existing environment | `environment_open` |
| List files | `environment_file_list` |
| Read file | `environment_file_read` |
| Write file | `environment_file_write` |
| Edit file | `environment_file_edit` |
| Run command | `environment_run_cmd` |
| Save progress | `environment_checkpoint` |
| **Delete environment** | `container-use delete <env_id>` (CLI) |
| **Update environments.json** | `Read` + `Edit` tools on `.opencode/environments.json` |

### Environment Naming Convention

```
<type>-<issue>-<feature>
```

Examples:
- `feature-issue-8-sound-playback`
- `fix-issue-6-ci-failure`
- `refactor-notification-module`

---

## Related Documents

| Document | Purpose | When to Reference |
|----------|---------|-------------------|
| [Design Sync Policy](./design-sync.md) | Keep design docs and implementation in sync | Before/during/after implementation |
| [Testing Strategy](./testing-strategy.md) | Handle environment-dependent code testing | When writing tests for OS/hardware-dependent code |
| [Platform Exception Policy](./platform-exception.md) | Platform-specific code exception rules | When implementing macOS/Windows-specific code |
| [container-use Guide](../skill/container-use-guide.md) | Step-by-step container environment setup | First time using container-use |

---

## 変更履歴

| 日付 | バージョン | 変更内容 |
|:---|:---|:---|
| 2026-01-07 | 3.14.0 | Session Summary Auto-Save セクションを追加。Supermemory との連携による自動復旧機能を追加。Related Documents に Platform Exception Policy を追加 |
| 2026-01-05 | 3.13.0 | environments.json必須化 |
| 2026-01-05 | 3.12.0 | 追加仕様対応 |
