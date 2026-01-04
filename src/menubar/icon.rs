use tray_icon::Icon;
use crate::types::{TimerState, TimerPhase};
use super::MenubarError;

pub struct IconManager {
    working_icon: Icon,
    breaking_icon: Icon,
    stopped_icon: Icon,
}

impl IconManager {
    pub fn new() -> Result<Self, MenubarError> {
        let default_icon = Self::create_default_icon()?;
        
        Ok(Self {
            working_icon: default_icon.clone(),
            breaking_icon: default_icon.clone(),
            stopped_icon: default_icon,
        })
    }

    pub fn generate_title(state: &TimerState) -> String {
        let minutes = state.remaining_seconds / 60;
        let seconds = state.remaining_seconds % 60;

        match state.phase {
            TimerPhase::Working => format!("🍅 {:02}:{:02}", minutes, seconds),
            TimerPhase::Breaking | TimerPhase::LongBreaking => {
                format!("☕ {:02}:{:02}", minutes, seconds)
            }
            TimerPhase::Paused => "⏸ 一時停止".to_string(),
            TimerPhase::Stopped => "⏸ 停止中".to_string(),
        }
    }

    pub fn get_icon(&self, phase: &TimerPhase) -> &Icon {
        match phase {
            TimerPhase::Working => &self.working_icon,
            TimerPhase::Breaking | TimerPhase::LongBreaking => &self.breaking_icon,
            TimerPhase::Stopped | TimerPhase::Paused => &self.stopped_icon,
        }
    }

    fn create_default_icon() -> Result<Icon, MenubarError> {
        let width = 22u32;
        let height = 22u32;
        // RGBA: 4 bytes per pixel. Gray color.
        let rgba: Vec<u8> = [128, 128, 128, 255].repeat((width * height) as usize);
        
        Icon::from_rgba(rgba, width, height)
            .map_err(|e| MenubarError::BuildError(e.to_string()))
    }
}
