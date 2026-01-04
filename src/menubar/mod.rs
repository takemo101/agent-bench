pub mod event;
pub mod icon;
pub mod menu;

use thiserror::Error;
use tray_icon::TrayIcon;

#[derive(Debug, Error)]
pub enum MenubarError {
    #[error("Failed to build tray icon: {0}")]
    BuildError(String),
    #[error("Tray icon not initialized")]
    NotInitialized,
    #[error("Menu error: {0}")]
    MenuError(String),
    #[error("Update error: {0}")]
    UpdateError(String),
}

pub struct TrayIconManager {
    // 骨格実装のため、フィールドは最小限かつunused警告を抑制
    #[allow(dead_code)]
    _tray_icon: Option<TrayIcon>,
}

impl Default for TrayIconManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TrayIconManager {
    pub fn new() -> Self {
        unimplemented!("TrayIconManager::new")
    }

    pub fn initialize(&mut self) -> Result<(), MenubarError> {
        unimplemented!("TrayIconManager::initialize")
    }

    pub fn update_state(&mut self) -> Result<(), MenubarError> {
        unimplemented!("TrayIconManager::update_state")
    }

    pub fn shutdown(&mut self) -> Result<(), MenubarError> {
        unimplemented!("TrayIconManager::shutdown")
    }
}
