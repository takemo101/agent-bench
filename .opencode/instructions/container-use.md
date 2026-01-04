# Container-Use Agent Rules

## Core Rules (MANDATORY - NO EXCEPTIONS)

ALWAYS use ONLY Environments for ANY and ALL file, code, or shell operations—NO EXCEPTIONS—even for simple or generic requests.

DO NOT install or use the git cli with the environment_run_cmd tool. All environment tools will handle git operations for you. Changing ".git" yourself will compromise the integrity of your environment.

You MUST inform the user how to view your work using `container-use log <env_id>` AND `container-use checkout <env_id>`. Failure to do this will make your work inaccessible to others.

---

## When to Use Container-Use

| Use Container-Use | Do NOT Use |
|-------------------|------------|
| Issue implementation (code changes) | Research / investigation only |
| New feature development | Documentation review |
| Bug fixes | Design discussions / reviews |
| Refactoring | Reading existing code |

**Decision criteria**: Will you modify files? → YES → Container-Use

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
   a. Create a NEW environment with the same branch
   b. The git state will be preserved from the remote
   c. Continue work in the new environment
   ```

---

## Error Handling Escalation

1. **First failure**: Retry the operation
2. **Second failure**: Check environment state, diagnose issue
3. **Third failure**: Create checkpoint, attempt alternative approach
4. **Persistent failure**: Report to user WITH environment ID for manual recovery

NEVER silently switch to non-container-use operations.

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
- [ ] PR created
- [ ] **CI passed** (MUST wait: `gh pr checks <PR番号> --watch`)
- [ ] PR merged (only AFTER CI passes)
- [ ] Issue closed
- [ ] Environment deleted: `container-use delete <env_id>` (after PR merge)

### PR Merge Flow (MANDATORY)

```bash
# 1. Create PR
gh pr create --title "..." --body "..."

# 2. Wait for CI to complete (NEVER skip this step)
gh pr checks <PR番号> --watch

# 3. Merge only after CI passes
gh pr merge <PR番号> --squash --delete-branch

# 4. Close related issue
gh issue close <Issue番号>
```

**HARD BLOCK**: Never merge a PR without confirming CI success.

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

### Environment Naming Convention

```
<type>-<issue>-<feature>
```

Examples:
- `feature-issue-8-sound-playback`
- `fix-issue-6-ci-failure`
- `refactor-notification-module`
