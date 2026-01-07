# Design Document Synchronization Policy

## Overview

This document defines the policy for keeping design documents and implementations in sync throughout the development lifecycle.

---

## When This Policy Applies

| Workflow Phase | Applies? | Notes |
|----------------|----------|-------|
| `/implement-issues` | **YES** | Always check design before implementing |
| `/detailed-design-workflow` | NO | Design is being created |
| Bug fixes (no design doc) | PARTIAL | Document if fix changes public API |
| Refactoring | **YES** | Verify design still matches after refactor |

**Integration with container-use workflow**:
- This policy is enforced during the PR creation step
- PR Description Template includes "Design Document Alignment" section
- See [container-use.md](./container-use.md) → PR Description Template

---

## Core Principles

| Principle | Description |
|-----------|-------------|
| **Design First** | Read design docs BEFORE implementation |
| **Document Deviations** | Any deviation from design MUST be documented |
| **Update or Annotate** | Either update the design doc OR annotate in PR |

---

## Implementation Phases

### Phase 1: Pre-Implementation

Before starting implementation:

1. **Read the design document thoroughly**
   - Identify API signatures, data structures, error handling
   - Note any ambiguities or questions

2. **Verify design completeness**
   - If design is incomplete, raise with user before proceeding
   - Do NOT make assumptions about unspecified behavior

3. **Check for conflicts**
   - Review existing code that may conflict with design
   - Identify integration points

### Phase 2: During Implementation

While implementing:

1. **Follow design specifications**
   - Use specified naming conventions
   - Implement specified API signatures
   - Follow specified error handling patterns

2. **Track deviations as they occur**
   - If deviation is necessary, document immediately
   - Include reasoning for the deviation

3. **Deviation categories**

   | Category | Examples | Action Required |
   |----------|----------|-----------------|
   | **Naming** | `UserStatus` → `AccountStatus` | Note in PR |
   | **API Shape** | Additional parameters, changed return types | Update design OR note in PR |
   | **Architecture** | Different module structure | Update design AND note in PR |
   | **Behavior** | Different error handling, edge cases | Update design AND note in PR |

### Phase 3: Post-Implementation

After implementation is complete:

1. **Review deviations list**
   - Categorize as intentional vs. accidental
   - Accidental deviations should be fixed

2. **Update or annotate**

   | Deviation Type | Action |
   |----------------|--------|
   | Minor (naming, formatting) | Note in PR description |
   | Moderate (API changes) | Update design doc OR detailed PR note |
   | Major (architecture) | MUST update design doc |

3. **PR description template includes design alignment section**
   - See `container-use.md` → PR Description Template

---

## Design Document Types

| Document Type | Location | Update Frequency |
|---------------|----------|------------------|
| Requirements | `docs/requirements/` | Rarely (scope changes only) |
| Basic Design | `docs/designs/basic/` | When architecture changes |
| Detailed Design | `docs/designs/detailed/` | When API/structure changes |
| Technical Research | `docs/research/` | When technology updates |

---

## Deviation Documentation Format

When documenting deviations in PR:

```markdown
### Deviations from Design

| Design Spec | Implementation | Reason |
|-------------|----------------|--------|
| `UserStatus` struct | `AccountStatus` struct | More generic naming for future extensibility |
| `fn get_status() -> Status` | `fn get_status() -> Result<AccountStatus, Error>` | Added error handling for edge cases |
```

---

## When to Update Design vs. Annotate PR

### Update Design Document

- Architecture changed significantly
- New public API that others will use
- New error types or handling patterns
- Changes affect multiple components

### Annotate in PR Only

- Minor naming differences
- Implementation details not in design
- Performance optimizations
- Bug fixes not covered in design

---

## Anti-Patterns

| Anti-Pattern | Why It's Bad | Correct Approach |
|--------------|--------------|------------------|
| Ignoring design doc | Wastes design effort, causes inconsistency | Read before implementing |
| Silent deviation | Others expect design to match code | Document all deviations |
| Over-updating design | Design becomes implementation log | Only update for significant changes |
| Blocking on design gaps | Slows velocity | Ask user, make reasonable assumption, document |

---

## Design Document Update Procedure

When updating design documents after implementation:

### Step 1: Identify What Changed

```bash
# Compare implementation with design
# Check: API signatures, data structures, module structure
```

### Step 2: Update the Document

| Document Type | Location | Update Method |
|---------------|----------|---------------|
| Detailed Design | `docs/designs/detailed/<feature>/` | Edit directly, update relevant sections |
| Basic Design | `docs/designs/basic/` | Only if architecture changed significantly |
| Requirements | `docs/requirements/` | Only if scope changed (rare) |

### Step 3: Mark Updated Sections

Add a note at the updated section:

```markdown
> **Updated**: 2026-01-04 - Changed `UserStatus` to `AccountStatus` per PR #54
```

### Step 4: Cross-Reference in PR

Include in PR description:

```markdown
## Design Document Updates
- Updated `docs/designs/detailed/{feature-name}/{component}.md`
  - Section 3.2: Changed struct name
  - Section 4.1: Added error handling
```

### When NOT to Update Design

- Implementation details not specified in design
- Performance optimizations
- Internal refactoring that doesn't change public API
- Bug fixes for edge cases

---

## Quick Reference

```
Before Implementation:
  1. Read design doc
  2. Note questions/ambiguities
  3. Ask if unclear

During Implementation:
  1. Follow design spec
  2. Track deviations in real-time
  3. Note reasoning

After Implementation:
  1. Review deviations
  2. Update design OR annotate PR
  3. Include in PR description
```

---

## Design Divergence Check (Simplified)

### 概要

設計書と実装の乖離は**手動チェック + PR記載**で管理します。
完全自動検出は設計書フォーマットの標準化が前提となるため、現時点では手動確認を推奨。

### チェックタイミング

| タイミング | 担当 | アクション |
|-----------|------|----------|
| 実装完了時 | 実装者 | 設計書を再読し、乖離があれば PR に記載 |
| レビュー時 | レビューエージェント | 設計書と実装の整合性をスコアに反映 |
| 定期 | 開発者 | `/reverse-engineer` で設計書を再生成、差分確認 |

### 実装者の責任

実装完了時に以下を確認：

```markdown
## PR作成前チェックリスト

- [ ] 設計書に記載された API シグネチャと一致しているか
- [ ] 設計書に記載されたデータ構造と一致しているか
- [ ] 乖離がある場合、PR本文の「Deviations from Design」セクションに記載したか
```

### 乖離を発見した場合

PR本文に明記：

```markdown
### Deviations from Design

| 設計書 | 実装 | 理由 |
|--------|------|------|
| `fn new(config: Config)` | `fn new(config: Config, executor: HookExecutor)` | フック実行機能を追加したため |
```

**選択肢**:
1. **設計書を更新** - アーキテクチャに影響する変更の場合（推奨）
2. **PR本文に記載のみ** - 軽微な変更の場合

### 定期チェック（推奨）

大規模な実装後は `/reverse-engineer` で設計書を再生成：

```bash
# 現在の実装から設計書を逆生成
/reverse-engineer "src/daemon/"

# 既存設計書との差分を確認（目視）
```

### 将来の自動化について

設計書フォーマットを YAML/JSON で標準化した場合、自動検出が可能になります。
現時点では手動チェックを推奨。

---

## Related Documents

| Document | Purpose |
|----------|---------|
| [container-use.md](./container-use.md) | PR creation workflow, completion criteria |
| [testing-strategy.md](./testing-strategy.md) | Test implementation guidelines |
| [platform-exception.md](./platform-exception.md) | Platform-specific code exception policy |

---

## 変更履歴

| 日付 | バージョン | 変更内容 |
|:---|:---|:---|
| 2026-01-07 | 1.1.0 | Automatic Divergence Detection セクションを追加 |
| 2026-01-04 | 1.0.0 | 初版作成 |
