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
   | **Naming** | `LaunchAgentStatus` → `ServiceStatus` | Note in PR |
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
| `LaunchAgentStatus` struct | `ServiceStatus` struct | More generic naming for future extensibility |
| `fn get_status() -> Status` | `fn get_status() -> Result<ServiceStatus>` | Added error handling for launchctl failures |
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
> **Updated**: 2026-01-04 - Changed `LaunchAgentStatus` to `ServiceStatus` per PR #54
```

### Step 4: Cross-Reference in PR

Include in PR description:

```markdown
## Design Document Updates
- Updated `docs/designs/detailed/pomodoro-timer/launch-agent.md`
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

## Automatic Divergence Detection (NEW)

### 概要

実装完了後、設計書との乖離を**自動検出**するチェックを実行します。
これにより、設計書の陳腐化を防ぎ、ドキュメントと実装の一貫性を保ちます。

### 検出タイミング

| タイミング | 検出方法 | アクション |
|-----------|---------|----------|
| PR 作成前 | `check_design_divergence()` | 乖離があれば警告 |
| 品質レビュー時 | レビューエージェントが設計書参照 | スコアに反映 |
| PR マージ後 | `/reverse-engineer` による定期チェック | 設計書更新提案 |

### 検出項目

| 項目 | 検出方法 | 重大度 |
|------|---------|--------|
| **API シグネチャ** | 関数名、引数、戻り値の比較 | 高 |
| **データ構造** | struct/enum のフィールド比較 | 高 |
| **モジュール構造** | ファイル配置、公開 API | 中 |
| **エラー型** | エラーバリアント、エラーコード | 中 |
| **命名規則** | 型名、関数名の一致 | 低 |

### 検出ロジック

```python
def check_design_divergence(issue_id: int, design_doc_path: str, changed_files: list[str]) -> DivergenceReport:
    """設計書と実装の乖離を検出"""
    
    divergences = []
    
    # 1. 設計書から API 仕様を抽出
    design_spec = extract_api_spec_from_design(design_doc_path)
    
    # 2. 実装から実際の API を抽出
    for file_path in changed_files:
        impl_spec = extract_api_spec_from_code(file_path)
        
        # 3. 比較
        # 関数シグネチャ
        for func_name, design_sig in design_spec.functions.items():
            impl_sig = impl_spec.functions.get(func_name)
            if impl_sig and impl_sig != design_sig:
                divergences.append(Divergence(
                    type="function_signature",
                    severity="high",
                    design=design_sig,
                    implementation=impl_sig,
                    message=f"関数 `{func_name}` のシグネチャが設計と異なります"
                ))
        
        # 構造体フィールド
        for struct_name, design_fields in design_spec.structs.items():
            impl_fields = impl_spec.structs.get(struct_name)
            if impl_fields and set(impl_fields) != set(design_fields):
                divergences.append(Divergence(
                    type="struct_fields",
                    severity="high",
                    design=design_fields,
                    implementation=impl_fields,
                    message=f"構造体 `{struct_name}` のフィールドが設計と異なります"
                ))
    
    return DivergenceReport(divergences=divergences)
```

### 乖離検出時のアクション

| 重大度 | アクション |
|--------|----------|
| **高** | PR 作成をブロック、ユーザーに確認を要求 |
| **中** | 警告を表示、PR 本文に乖離セクションを追加 |
| **低** | PR 本文に記載のみ |

#### 高重大度の乖離が検出された場合

```markdown
## ⚠️ 設計書との乖離を検出

以下の乖離が検出されました。PR 作成前に確認してください。

| 項目 | 設計書 | 実装 |
|------|--------|------|
| `TimerEngine::new()` | `fn new(config: Config) -> Self` | `fn new(config: Config, executor: HookExecutor) -> Self` |

**選択肢**:
1. **設計書を更新**: 実装に合わせて設計書を修正
2. **実装を修正**: 設計書に合わせて実装を修正
3. **乖離を承認**: 意図的な乖離として PR に記載して続行

どれを選択しますか？
```

### 定期チェック（推奨）

大規模な実装後は `/reverse-engineer` を実行して設計書を再生成し、
既存の設計書と比較することを推奨します。

```bash
# 現在の実装から設計書を逆生成
/reverse-engineer "src/daemon/"

# 既存設計書との差分を確認
diff docs/designs/detailed/pomodoro-timer/daemon-server.md \
     docs/designs/reverse/pomodoro-timer-current.md
```

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
