//! Visual Enhancement Integration Tests
//!
//! Tests for the visual enhancement feature (F-035, F-036, F-037).
//! Verifies integration between TimeDisplay, AnimationEngine, LayoutRenderer,
//! TerminalController, and EnhancedDisplayState.
//!
//! Test Spec Reference: docs/designs/detailed/visual-enhancement/共通/test-specification.md

use pomodoro::cli::animation::{AnimationEngine, AnimationFrame};
use pomodoro::cli::display::EnhancedDisplayState;
use pomodoro::cli::layout::{DisplayLayout, LayoutRenderer};
use pomodoro::cli::time_format::TimeDisplay;
use pomodoro::types::TimerPhase;

// ============================================================================
// INT-001, INT-002: TimeFormatter + LayoutRenderer Integration
// ============================================================================

/// INT-001: TimeDisplay integrates correctly with LayoutRenderer
#[test]
fn test_time_display_layout_integration() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(323, 1500);

    let layout = renderer.build_layout(
        TimerPhase::Working,
        &time_display,
        None,
        None,
        323,
        1500,
    );

    // line1 should contain the formatted time
    assert!(layout.line1.contains("05:23"));
    assert!(layout.line1.contains("25:00"));
    assert!(layout.line1.contains("21%"));
}

/// INT-002: Percentage is correctly reflected in layout
#[test]
fn test_percentage_in_layout() {
    let renderer = LayoutRenderer::new(80);

    // 50% progress
    let time_display = TimeDisplay::new(750, 1500);
    let layout = renderer.build_layout(
        TimerPhase::Working,
        &time_display,
        None,
        None,
        750,
        1500,
    );

    assert!(layout.line1.contains("50%"));
}

/// Test various time display formats
#[test]
fn test_time_display_formats() {
    // TF-001: Normal time display
    let td = TimeDisplay::new(323, 1500);
    assert_eq!(td.format(), "05:23/25:00 (21%)");

    // TF-002: Start
    let td = TimeDisplay::new(0, 1500);
    assert_eq!(td.format(), "00:00/25:00 (0%)");

    // TF-003: Complete
    let td = TimeDisplay::new(1500, 1500);
    assert_eq!(td.format(), "25:00/25:00 (100%)");

    // TF-005: Short break
    let td = TimeDisplay::new(150, 300);
    assert_eq!(td.format(), "02:30/05:00 (50%)");

    // TF-006: Long break
    let td = TimeDisplay::new(450, 900);
    assert_eq!(td.format(), "07:30/15:00 (50%)");
}

// ============================================================================
// INT-003, INT-004: AnimationEngine + LayoutRenderer Integration
// ============================================================================

/// INT-003: AnimationFrame integrates correctly with LayoutRenderer
#[test]
fn test_animation_frame_layout_integration() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(0, 1500);
    let frame = AnimationFrame::new("🏃💨 ─────────────────────────────");

    let layout = renderer.build_layout(
        TimerPhase::Working,
        &time_display,
        Some(&frame),
        None,
        0,
        1500,
    );

    // line2 should contain the animation frame
    assert!(layout.line2.contains("🏃"));
    assert!(layout.line2.contains("💨"));
}

/// INT-004: Animation changes on phase transition
#[test]
fn test_animation_phase_transition() {
    let engine = AnimationEngine::new();

    // Get Working animation frame
    let work_frame = engine.get_current_frame(TimerPhase::Working);
    assert!(work_frame.is_some());

    // Get Breaking animation frame
    let break_frame = engine.get_current_frame(TimerPhase::Breaking);
    assert!(break_frame.is_some());

    // Frames should be different
    assert_ne!(work_frame, break_frame);
}

/// Test animation engine tick advances frames
#[test]
fn test_animation_engine_tick() {
    let mut engine = AnimationEngine::new();

    let frame1 = engine.get_current_frame(TimerPhase::Working).unwrap();
    engine.tick();
    let frame2 = engine.get_current_frame(TimerPhase::Working).unwrap();

    // Frames should be different after tick
    assert_ne!(frame1, frame2);
}

/// Test animation reset on phase change
#[test]
fn test_animation_reset() {
    let mut engine = AnimationEngine::new();

    // Advance a few ticks
    engine.tick();
    engine.tick();
    engine.tick();

    // Reset
    engine.reset();

    // Should be back to frame 0
    let frame_after_reset = engine.get_current_frame(TimerPhase::Working).unwrap();
    let fresh_engine = AnimationEngine::new();
    let frame_fresh = fresh_engine.get_current_frame(TimerPhase::Working).unwrap();

    assert_eq!(frame_after_reset, frame_fresh);
}

// ============================================================================
// INT-005, INT-006, INT-007: LayoutRenderer + TerminalController Integration
// ============================================================================

/// INT-005: DisplayLayout renders without error
#[test]
fn test_display_layout_render() {
    let layout = DisplayLayout::new(
        "🍅 作業中 [████████░░░░] 05:23/25:00 (21%)".to_string(),
        "🏃💨 ─────────────────────────────".to_string(),
        Some("タスク: コーディング".to_string()),
    );

    // Should have 3 lines
    assert_eq!(layout.line_count, 3);
    assert_eq!(layout.lines().len(), 3);
}

/// INT-007: Line count changes are handled correctly
#[test]
fn test_layout_line_count_changes() {
    // 3 lines with task
    let layout3 = DisplayLayout::new(
        "Line 1".to_string(),
        "Line 2".to_string(),
        Some("Line 3".to_string()),
    );
    assert_eq!(layout3.line_count, 3);

    // 2 lines without task
    let layout2 = DisplayLayout::new("Line 1".to_string(), "Line 2".to_string(), None);
    assert_eq!(layout2.line_count, 2);
}

// ============================================================================
// EnhancedDisplayState Integration Tests
// ============================================================================

/// Test EnhancedDisplayState initialization
#[test]
fn test_enhanced_display_state_init() {
    let state = EnhancedDisplayState::new();
    assert!(state.current_phase.is_none());
}

/// Test phase change detection in EnhancedDisplayState
#[test]
fn test_enhanced_display_state_phase_change() {
    let mut state = EnhancedDisplayState::new();

    // Initial phase should be None
    assert!(state.current_phase.is_none());

    // After update, phase should be set
    // Note: We can't fully test update() without a real terminal,
    // but we can test the phase tracking logic
    state.current_phase = Some(TimerPhase::Working);
    assert_eq!(state.current_phase, Some(TimerPhase::Working));

    // Phase change
    state.current_phase = Some(TimerPhase::Breaking);
    assert_eq!(state.current_phase, Some(TimerPhase::Breaking));
}

// ============================================================================
// Layout Structure Tests (LR-001 to LR-005)
// ============================================================================

/// LR-001: 3-line layout with task
#[test]
fn test_layout_3_lines_with_task() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(323, 1500);
    let frame = AnimationFrame::new("🏃💨 ─────");

    let layout = renderer.build_layout(
        TimerPhase::Working,
        &time_display,
        Some(&frame),
        Some("コーディング"),
        323,
        1500,
    );

    assert_eq!(layout.line_count, 3);
    assert!(layout.line3.is_some());
    assert!(layout.line3.as_ref().unwrap().contains("タスク"));
}

/// LR-002: 2-line layout without task
#[test]
fn test_layout_2_lines_without_task() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(323, 1500);
    let frame = AnimationFrame::new("🏃💨 ─────");

    let layout = renderer.build_layout(
        TimerPhase::Working,
        &time_display,
        Some(&frame),
        None,
        323,
        1500,
    );

    assert_eq!(layout.line_count, 2);
    assert!(layout.line3.is_none());
}

/// LR-003: Progress bar construction
#[test]
fn test_progress_bar_construction() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(750, 1500); // 50%

    let layout = renderer.build_layout(TimerPhase::Working, &time_display, None, None, 750, 1500);

    // line1 should contain progress bar characters
    assert!(layout.line1.contains('█') || layout.line1.contains('░'));
}

/// LR-004: Phase style mapping
#[test]
fn test_phase_style_mapping() {
    // Working
    let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::Working);
    assert_eq!(icon, "🍅");
    assert_eq!(label, "作業中");
    assert_eq!(color, "red");

    // Breaking
    let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::Breaking);
    assert_eq!(icon, "☕");
    assert_eq!(label, "休憩中");
    assert_eq!(color, "green");

    // LongBreaking
    let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::LongBreaking);
    assert_eq!(icon, "🛏️");
    assert_eq!(label, "長期休憩中");
    assert_eq!(color, "blue");

    // Paused
    let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::Paused);
    assert_eq!(icon, "⏸️");
    assert_eq!(label, "一時停止");
    assert_eq!(color, "yellow");

    // Stopped
    let (icon, label, color) = LayoutRenderer::phase_style(TimerPhase::Stopped);
    assert_eq!(icon, "⏹");
    assert_eq!(label, "停止");
    assert_eq!(color, "white");
}

/// LR-005: Terminal width affects bar width
#[test]
fn test_terminal_width_bar_width() {
    // Narrow terminal
    let renderer_narrow = LayoutRenderer::new(40);
    assert_eq!(renderer_narrow.bar_width(), 16); // 40 * 0.4 = 16

    // Standard terminal
    let renderer_std = LayoutRenderer::new(80);
    assert_eq!(renderer_std.bar_width(), 32); // 80 * 0.4 = 32

    // Wide terminal (capped at 40)
    let renderer_wide = LayoutRenderer::new(120);
    assert_eq!(renderer_wide.bar_width(), 40); // capped
}

// ============================================================================
// All Phases Layout Test
// ============================================================================

/// Test layout generation for all phases
#[test]
fn test_all_phases_layout() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(300, 1500);

    let phases = [
        TimerPhase::Working,
        TimerPhase::Breaking,
        TimerPhase::LongBreaking,
        TimerPhase::Paused,
        TimerPhase::Stopped,
    ];

    for phase in phases {
        let layout = renderer.build_layout(phase, &time_display, None, None, 300, 1500);

        // All phases should produce valid layout
        assert!(!layout.line1.is_empty());
        // line2 is empty when no animation frame is provided
        assert_eq!(layout.line_count, 2);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

/// Test with zero duration
#[test]
fn test_zero_duration() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(0, 0);

    let layout =
        renderer.build_layout(TimerPhase::Stopped, &time_display, None, None, 0, 0);

    // Should not panic and produce valid layout
    assert!(!layout.line1.is_empty());
}

/// Test with elapsed > total (overtime)
#[test]
fn test_overtime() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(1600, 1500);

    let layout =
        renderer.build_layout(TimerPhase::Working, &time_display, None, None, 1600, 1500);

    // Should handle gracefully
    assert!(layout.line1.contains("100%"));
}

/// Test with very long task name
#[test]
fn test_long_task_name() {
    let renderer = LayoutRenderer::new(80);
    let time_display = TimeDisplay::new(0, 1500);
    let long_task = "これは非常に長いタスク名で、ターミナルの幅を超える可能性があります。";

    let layout = renderer.build_layout(
        TimerPhase::Working,
        &time_display,
        None,
        Some(long_task),
        0,
        1500,
    );

    // Should include the task name (truncation is not implemented yet)
    assert!(layout.line3.is_some());
    assert!(layout.line3.as_ref().unwrap().contains("タスク"));
}

/// Test unicode width calculation in animation frame
#[test]
fn test_animation_frame_unicode_width() {
    // Japanese text has width 2 per character
    let frame = AnimationFrame::new("日本語");
    assert_eq!(frame.width, 6); // 3 chars * 2 width each

    // Emoji width (typically 2)
    let frame_emoji = AnimationFrame::new("🏃💨");
    // Width depends on unicode-width implementation
    assert!(frame_emoji.width > 0);
}
