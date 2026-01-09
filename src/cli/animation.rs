use crate::types::TimerPhase;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

/// アニメーションの1フレーム
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationFrame {
    pub content: String,
    pub width: usize,
}

impl AnimationFrame {
    /// 新しいフレームを作成
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let width = content.width();
        Self { content, width }
    }

    /// 指定された幅にパディング（中央寄せ）
    pub fn padded(&self, target_width: usize) -> String {
        if self.width >= target_width {
            return self.content.clone();
        }
        
        let total_padding = target_width - self.width;
        let left_padding = total_padding / 2;
        let right_padding = total_padding - left_padding;
        
        format!("{}{}{}", " ".repeat(left_padding), self.content, " ".repeat(right_padding))
    }
}

/// フェーズごとのアニメーション定義
#[derive(Debug, Clone)]
pub struct PhaseAnimation {
    pub phase: TimerPhase,
    pub frames: Vec<AnimationFrame>,
    pub fps: u64,
}

impl PhaseAnimation {
    /// 作業中アニメーション
    pub fn work() -> Self {
        let frames = vec![
            AnimationFrame::new("🏃💨 ─────────────────────────────"),
            AnimationFrame::new(" 🏃💨 ────────────────────────────"),
            AnimationFrame::new("  🏃💨 ───────────────────────────"),
            AnimationFrame::new("   🏃💨 ──────────────────────────"),
        ];
        Self {
            phase: TimerPhase::Working,
            frames,
            fps: 5,
        }
    }

    /// 休憩中アニメーション
    pub fn short_break() -> Self {
        let frames = vec![
            AnimationFrame::new("🧘 ～～～ ゆっくり休憩中 ～～～"),
            AnimationFrame::new("🧘  ～～～ ゆっくり休憩中 ～～～"),
            AnimationFrame::new("🧘 ～～～  ゆっくり休憩中 ～～～"),
            AnimationFrame::new("🧘  ～～～ ゆっくり休憩中  ～～～"),
        ];
        Self {
            phase: TimerPhase::Breaking,
            frames,
            fps: 5,
        }
    }

    /// 長期休憩中アニメーション
    pub fn long_break() -> Self {
        let frames = vec![
            AnimationFrame::new("😴💤 zzz... ───────────────────"),
            AnimationFrame::new("😴💤  zzz... ──────────────────"),
        ];
        Self {
            phase: TimerPhase::LongBreaking,
            frames,
            fps: 5,
        }
    }

    /// 一時停止中アニメーション
    pub fn paused() -> Self {
        let frames = vec![
            AnimationFrame::new("   （一時停止中）   "),
        ];
        Self {
            phase: TimerPhase::Paused,
            frames,
            fps: 1,
        }
    }
}

/// アニメーションエンジン
#[derive(Debug)]
pub struct AnimationEngine {
    animations: HashMap<TimerPhase, PhaseAnimation>,
    frame_counter: usize,
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationEngine {
    pub fn new() -> Self {
        let mut animations = HashMap::new();
        animations.insert(TimerPhase::Working, PhaseAnimation::work());
        animations.insert(TimerPhase::Breaking, PhaseAnimation::short_break());
        animations.insert(TimerPhase::LongBreaking, PhaseAnimation::long_break());
        animations.insert(TimerPhase::Paused, PhaseAnimation::paused());
        
        Self {
            animations,
            frame_counter: 0,
        }
    }

    pub fn tick(&mut self) {
        self.frame_counter += 1;
    }

    pub fn get_current_frame(&self, phase: TimerPhase) -> Option<String> {
        if phase == TimerPhase::Stopped {
            return None;
        }
        
        let animation = self.animations.get(&phase)?;
        if animation.frames.is_empty() {
            return None;
        }
        
        let index = self.frame_counter % animation.frames.len();
        Some(animation.frames[index].content.clone())
    }
    
    pub fn reset(&mut self) {
        self.frame_counter = 0;
    }
    
    pub fn interval_ms(&self, _phase: TimerPhase) -> u64 {
        // 全フェーズ共通で200ms
        200
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_frame_new() {
        let frame = AnimationFrame::new("test");
        assert_eq!(frame.content, "test");
        assert_eq!(frame.width, 4);

        let wide_frame = AnimationFrame::new("テスト");
        assert_eq!(wide_frame.content, "テスト");
        assert_eq!(wide_frame.width, 6);
    }

    #[test]
    fn test_animation_frame_padded() {
        let frame = AnimationFrame::new("test"); // width 4
        // target 6: " test " (left 1, right 1)
        assert_eq!(frame.padded(6), " test ");
        // target 7: " test  " (left 1, right 2)
        assert_eq!(frame.padded(7), " test  ");
    }
    
    #[test]
    fn test_phase_animation_factories() {
        let work = PhaseAnimation::work();
        assert_eq!(work.phase, TimerPhase::Working);
        assert_eq!(work.frames.len(), 4);
        
        let br = PhaseAnimation::short_break();
        assert_eq!(br.phase, TimerPhase::Breaking);
        assert_eq!(br.frames.len(), 4);
        
        let lbr = PhaseAnimation::long_break();
        assert_eq!(lbr.phase, TimerPhase::LongBreaking);
        assert_eq!(lbr.frames.len(), 2);
        
        let paused = PhaseAnimation::paused();
        assert_eq!(paused.phase, TimerPhase::Paused);
        assert_eq!(paused.frames.len(), 1);
    }

    #[test]
    fn test_animation_engine_new() {
        let engine = AnimationEngine::new();
        assert_eq!(engine.frame_counter, 0);
        assert!(engine.animations.contains_key(&TimerPhase::Working));
    }

    #[test]
    fn test_animation_engine_tick_and_get() {
        let mut engine = AnimationEngine::new();
        let frame1 = engine.get_current_frame(TimerPhase::Working).unwrap();
        
        engine.tick();
        let frame2 = engine.get_current_frame(TimerPhase::Working).unwrap();
        
        assert_ne!(frame1, frame2);
        
        // 4フレームでループするので、3回さらにtickすると元に戻るはず
        engine.tick(); // 2
        engine.tick(); // 3
        engine.tick(); // 4 -> 0
        
        let frame5 = engine.get_current_frame(TimerPhase::Working).unwrap();
        assert_eq!(frame1, frame5);
    }
    
    #[test]
    fn test_animation_engine_reset() {
        let mut engine = AnimationEngine::new();
        engine.tick();
        assert_eq!(engine.frame_counter, 1);
        engine.reset();
        assert_eq!(engine.frame_counter, 0);
    }
    
    #[test]
    fn test_animation_engine_interval() {
        let engine = AnimationEngine::new();
        assert_eq!(engine.interval_ms(TimerPhase::Working), 200);
    }
    
    #[test]
    fn test_animation_engine_stopped() {
        let engine = AnimationEngine::new();
        let frame = engine.get_current_frame(TimerPhase::Stopped);
        assert!(frame.is_none());
    }
}
