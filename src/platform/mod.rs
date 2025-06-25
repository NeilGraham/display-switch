use anyhow::Result;
use crate::display::DisplayMode;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

// For unsupported platforms, provide a stub implementation
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
mod stub;

pub struct PlatformDisplayManager {
    #[cfg(target_os = "windows")]
    inner: windows::WindowsDisplayManager,
    #[cfg(target_os = "linux")]
    inner: linux::LinuxDisplayManager,
    #[cfg(target_os = "macos")]
    inner: macos::MacOSDisplayManager,
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    inner: stub::StubDisplayManager,
}

impl PlatformDisplayManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(target_os = "windows")]
            inner: windows::WindowsDisplayManager::new()?,
            #[cfg(target_os = "linux")]
            inner: linux::LinuxDisplayManager::new()?,
            #[cfg(target_os = "macos")]
            inner: macos::MacOSDisplayManager::new()?,
            #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
            inner: stub::StubDisplayManager::new()?,
        })
    }

    pub async fn get_available_modes(&self) -> Result<Vec<DisplayMode>> {
        #[cfg(target_os = "windows")]
        return self.inner.get_available_modes().await;
        #[cfg(target_os = "linux")]
        return self.inner.get_available_modes().await;
        #[cfg(target_os = "macos")]
        return self.inner.get_available_modes().await;
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        return self.inner.get_available_modes().await;
    }

    pub async fn set_display_mode(&self, mode: &DisplayMode) -> Result<()> {
        #[cfg(target_os = "windows")]
        return self.inner.set_display_mode(mode).await;
        #[cfg(target_os = "linux")]
        return self.inner.set_display_mode(mode).await;
        #[cfg(target_os = "macos")]
        return self.inner.set_display_mode(mode).await;
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        return self.inner.set_display_mode(mode).await;
    }

    pub async fn get_current_display_mode(&self) -> Result<DisplayMode> {
        #[cfg(target_os = "windows")]
        return self.inner.get_current_display_mode().await;
        #[cfg(target_os = "linux")]
        return self.inner.get_current_display_mode().await;
        #[cfg(target_os = "macos")]
        return self.inner.get_current_display_mode().await;
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        return self.inner.get_current_display_mode().await;
    }
} 