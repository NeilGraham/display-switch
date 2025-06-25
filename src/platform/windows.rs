use anyhow::{anyhow, Result};
use std::ffi::CString;
use std::mem;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::windef::{HDC, HMONITOR, LPRECT};
use winapi::um::wingdi::DEVMODEA;
use winapi::um::winuser::{
    ChangeDisplaySettingsA, EnumDisplaySettingsA, CDS_UPDATEREGISTRY, DISP_CHANGE_SUCCESSFUL,
};

use crate::display::DisplayMode;

pub struct WindowsDisplayManager;

impl WindowsDisplayManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub async fn get_available_modes(&self) -> Result<Vec<DisplayMode>> {
        let mut modes = Vec::new();
        let mut mode_index = 0;

        unsafe {
            loop {
                let mut dev_mode: DEVMODEA = mem::zeroed();
                dev_mode.dmSize = mem::size_of::<DEVMODEA>() as u16;

                let result = EnumDisplaySettingsA(
                    std::ptr::null(),
                    mode_index,
                    &mut dev_mode,
                );

                if result == 0 {
                    break;
                }

                // Skip modes with unusual bit depths
                if dev_mode.dmBitsPerPel >= 24 {
                    let mode = DisplayMode {
                        width: dev_mode.dmPelsWidth,
                        height: dev_mode.dmPelsHeight,
                        refresh_rate: dev_mode.dmDisplayFrequency as f64,
                    };

                    // Avoid duplicates and invalid modes
                    if mode.width > 0 && mode.height > 0 && mode.refresh_rate > 0.0 
                        && !modes.iter().any(|m: &DisplayMode| m.width == mode.width && m.height == mode.height && (m.refresh_rate - mode.refresh_rate).abs() < 0.1) {
                        modes.push(mode);
                    }
                }

                mode_index += 1;
            }
        }

        if modes.is_empty() {
            return Err(anyhow!("No display modes found"));
        }

        // Sort by resolution, then by refresh rate
        modes.sort_by(|a, b| {
            match (a.width * a.height).cmp(&(b.width * b.height)) {
                std::cmp::Ordering::Equal => a.refresh_rate.partial_cmp(&b.refresh_rate).unwrap(),
                other => other,
            }
        });

        Ok(modes)
    }

    pub async fn set_display_mode(&self, mode: &DisplayMode) -> Result<()> {
        unsafe {
            let mut dev_mode: DEVMODEA = mem::zeroed();
            dev_mode.dmSize = mem::size_of::<DEVMODEA>() as u16;
            dev_mode.dmPelsWidth = mode.width;
            dev_mode.dmPelsHeight = mode.height;
            dev_mode.dmDisplayFrequency = mode.refresh_rate as u32;
            dev_mode.dmBitsPerPel = 32; // Use 32-bit color depth
            dev_mode.dmFields = 0x00040000 | 0x00080000 | 0x00400000 | 0x00020000; // DM_PELSWIDTH | DM_PELSHEIGHT | DM_DISPLAYFREQUENCY | DM_BITSPERPEL

            let result = ChangeDisplaySettingsA(&mut dev_mode, CDS_UPDATEREGISTRY);

            if result != DISP_CHANGE_SUCCESSFUL {
                return Err(anyhow!(
                    "Failed to change display settings. Error code: {}",
                    result
                ));
            }
        }

        Ok(())
    }
}

// Callback function for enumerating monitors (for future multi-monitor support)
unsafe extern "system" fn monitor_enum_proc(
    _monitor: HMONITOR,
    _hdc: HDC,
    _rect: LPRECT,
    _data: LPARAM,
) -> i32 {
    1 // Continue enumeration
} 