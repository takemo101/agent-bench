# Testing Strategy for Environment-Dependent Code

## Overview

This document defines testing strategies for code that depends on specific environments (OS, hardware, external services).

---

## Related Issues

- **Issue #13**: [Test] コンポーネント間統合テスト - Integration tests for daemon-CLI, timer-notification
- **Issue #14**: [Test] E2Eテストとパフォーマンステスト - End-to-end and performance tests

This document provides guidelines for implementing these test issues.

---

## Current Project Test Structure

**Note**: The current project uses a flat structure. The recommended structure below is for new/refactored projects.

| Current (this project) | Recommended (new projects) |
|------------------------|---------------------------|
| `tests/test_*.rs` (flat) | `tests/unit/`, `tests/integration/`, `tests/e2e/` |
| `src/**/mod.rs` (inline tests) | Same (unit tests in modules) |

**Migration**: When implementing Issue #13/#14, create `tests/integration/` directory for new integration tests.

---

## Code Classification

| Category | Examples | Testing Strategy |
|----------|----------|------------------|
| **Pure Logic** | Data transformations, calculations, parsing | Standard unit tests |
| **OS API Calls** | launchctl, systemctl, Windows Registry | Mock or `#[ignore]` |
| **File System** | Config files, log files, plist files | tempdir or Mock |
| **Network** | HTTP clients, IPC sockets | Mock server or `#[ignore]` |
| **Hardware** | Audio devices, display, sensors | `#[ignore]` + manual testing |

---

## Rust-Specific Patterns

### Pattern 1: Trait Abstraction (Recommended)

Abstract OS-dependent operations behind traits for testability:

```rust
// Define trait for the operation
pub trait LaunchctlExecutor {
    fn load(&self, plist_path: &Path) -> Result<()>;
    fn unload(&self, plist_path: &Path) -> Result<()>;
    fn list(&self) -> Result<Vec<ServiceInfo>>;
}

// Production implementation
pub struct RealLaunchctl;

impl LaunchctlExecutor for RealLaunchctl {
    fn load(&self, plist_path: &Path) -> Result<()> {
        Command::new("launchctl")
            .args(["load", "-w", plist_path.to_str().unwrap()])
            .status()?;
        Ok(())
    }
    // ... other methods
}

// Test implementation
#[cfg(test)]
pub struct MockLaunchctl {
    pub load_result: Result<()>,
    pub unload_result: Result<()>,
    pub services: Vec<ServiceInfo>,
}

#[cfg(test)]
impl LaunchctlExecutor for MockLaunchctl {
    fn load(&self, _path: &Path) -> Result<()> {
        self.load_result.clone()
    }
    // ... other methods
}
```

### Pattern 2: Feature Flags

Use Cargo features to conditionally compile test implementations:

```toml
# Cargo.toml
[features]
default = []
test-mocks = []

[dev-dependencies]
# test dependencies
```

```rust
#[cfg(feature = "test-mocks")]
pub mod mocks {
    pub struct MockNotificationCenter { /* ... */ }
}
```

### Pattern 3: Conditional Test Attributes

For tests that require specific environments:

```rust
// Skip in CI, run locally on macOS
#[test]
#[ignore = "Requires macOS with launchctl"]
fn test_launchctl_integration() {
    // This test only runs with: cargo test -- --ignored
}

// Platform-specific test
#[test]
#[cfg(target_os = "macos")]
fn test_macos_specific_feature() {
    // Only compiled and run on macOS
}

// CI-aware test
#[test]
fn test_with_ci_awareness() {
    if std::env::var("CI").is_ok() {
        // Simplified test for CI
        return;
    }
    // Full test for local development
}
```

---

## Test Organization

### Directory Structure

```
tests/
├── unit/                    # Pure logic tests (always run)
│   ├── mod.rs
│   └── parser_test.rs
├── integration/             # Component integration (may need mocks)
│   ├── mod.rs
│   ├── daemon_cli.rs
│   └── timer_notification.rs
├── e2e/                     # End-to-end (environment-dependent)
│   ├── mod.rs
│   └── full_cycle.rs        # #[ignore] for CI
└── fixtures/                # Test data
    └── sample.plist
```

### Test Naming Convention

| Test Type | Naming Pattern | Example |
|-----------|----------------|---------|
| Unit | `test_<function>_<scenario>` | `test_parse_valid_input` |
| Integration | `test_<component>_<interaction>` | `test_service_sends_notification` |
| E2E | `test_<scenario>_flow` | `test_full_checkout_flow` |
| Ignored | Add `_requires_<env>` suffix | `test_native_api_requires_macos` |

---

## CI Configuration

### GitHub Actions Matrix

```yaml
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo test
      
      # Run ignored tests only on macOS
      - name: Run macOS-specific tests
        if: matrix.os == 'macos-latest'
        run: cargo test -- --ignored
```

### Test Categories in CI

| Category | CI Behavior | Local Behavior |
|----------|-------------|----------------|
| Unit tests | Always run | Always run |
| Integration (mocked) | Always run | Always run |
| Integration (real) | `#[ignore]` skip | Run with `--ignored` |
| E2E | `#[ignore]` skip | Run with `--ignored` |
| Performance | Separate job | Manual |

---

## Mock Implementation Guidelines

### What to Mock

| Component | Mock? | Reason |
|-----------|-------|--------|
| External commands (launchctl, systemctl) | YES | Not available in CI |
| File system (config files) | PARTIAL | Use tempdir for isolation |
| Network (IPC sockets) | YES | Avoid port conflicts |
| Time (delays, timers) | YES | Speed up tests |
| Audio playback | YES | No audio device in CI |
| Notifications | YES | No notification center in CI |

### What NOT to Mock

| Component | Reason |
|-----------|--------|
| Your own pure functions | Test the real thing |
| Data structures | Test the real thing |
| Serialization/deserialization | Test the real thing |
| Error types | Test the real thing |

---

## Recommended Mock Libraries

### Rust

| Library | Use Case | Notes |
|---------|----------|-------|
| **mockall** | Auto-generate mocks from traits | Most popular, macro-based |
| **mockito** | HTTP server mocking | For API client tests |
| **tempfile** | Temporary directories/files | For filesystem tests |
| **tokio-test** | Async runtime for tests | For async code |

**mockall Example**:

```rust
use mockall::{automock, predicate::*};

#[automock]
pub trait NotificationSender {
    fn send(&self, title: &str, body: &str) -> Result<(), Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_sends_notification() {
        let mut mock = MockNotificationSender::new();
        mock.expect_send()
            .with(eq("Work Complete"), always())
            .times(1)
            .returning(|_, _| Ok(()));

        // Use mock in test...
    }
}
```

**When to use mockall vs manual mocks**:

| Scenario | Recommendation |
|----------|----------------|
| Many methods on trait | Use mockall (less boilerplate) |
| Complex return types | Use mockall |
| Simple trait (1-2 methods) | Manual mock is fine |
| Need stateful mock | Manual mock gives more control |

---

## Documentation Requirements

### Test Documentation

Each test file should have a header comment:

```rust
//! # Timer Engine Tests
//!
//! ## Environment Requirements
//! - None (pure logic tests)
//!
//! ## Mock Dependencies
//! - `MockNotificationSender`
//! - `MockSoundPlayer`
//!
//! ## Ignored Tests
//! - `test_real_notification_*` - Requires macOS notification center
```

### Ignored Test Documentation

Each `#[ignore]` test must have a reason:

```rust
#[test]
#[ignore = "Requires macOS 13+ with Focus Mode enabled"]
fn test_focus_mode_activation() {
    // ...
}
```

---

## Quick Reference

### Running Tests

```bash
# All tests (skips #[ignore])
cargo test

# Include ignored tests
cargo test -- --ignored

# Only ignored tests
cargo test -- --ignored --include-ignored

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Test Attributes Cheat Sheet

```rust
#[test]                           // Standard test
#[ignore]                         // Skip by default
#[ignore = "reason"]              // Skip with reason
#[cfg(test)]                      // Only compile for tests
#[cfg(target_os = "macos")]       // Platform-specific
#[should_panic]                   // Expect panic
#[should_panic(expected = "msg")] // Expect specific panic
```

---

## Related Documents

| Document | Purpose |
|----------|---------|
| [container-use.md](./container-use.md) | Environment setup, PR workflow |
| [design-sync.md](./design-sync.md) | Design document synchronization |
