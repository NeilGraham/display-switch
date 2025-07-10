use anyhow::{anyhow, Result};
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{CFRelease, CFTypeRef, TCFType};
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::number::{CFNumber, CFNumberRef};
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::display::{
    CGDirectDisplayID, CGDisplayCopyAllDisplayModes, CGDisplayModeGetHeight,
    CGDisplayModeGetRefreshRate, CGDisplayModeGetWidth, CGDisplayModeRef, CGDisplaySetDisplayMode,
    CGGetActiveDisplayList, CGMainDisplayID,
};

use crate::display::DisplayMode;

pub struct MacOSDisplayManager {
    display_id: CGDirectDisplayID,
}

impl MacOSDisplayManager {
    pub fn new() -> Result<Self> {
        unsafe {
            let display_id = CGMainDisplayID();
            Ok(Self { display_id })
        }
    }

    pub async fn get_available_modes(&self) -> Result<Vec<DisplayMode>> {
        unsafe {
            let modes_array = CGDisplayCopyAllDisplayModes(self.display_id, std::ptr::null());
            if modes_array.is_null() {
                return Err(anyhow!("Failed to get display modes"));
            }

            let count = CFArray::wrap_under_create_rule(modes_array).len();
            let mut display_modes = Vec::new();

            for i in 0..count {
                let mode_ref = CFArray::wrap_under_create_rule(modes_array).get(i);
                if mode_ref.is_null() {
                    continue;
                }

                let mode_ref = mode_ref as CGDisplayModeRef;
                let width = CGDisplayModeGetWidth(mode_ref) as u32;
                let height = CGDisplayModeGetHeight(mode_ref) as u32;
                let refresh_rate = CGDisplayModeGetRefreshRate(mode_ref);

                let mode = DisplayMode {
                    width,
                    height,
                    refresh_rate,
                };

                // Avoid duplicates and filter out unusable modes
                if width > 0 && height > 0 && refresh_rate > 0.0 {
                    if !display_modes.iter().any(|m| {
                        m.width == mode.width
                            && m.height == mode.height
                            && (m.refresh_rate - mode.refresh_rate).abs() < 0.1
                    }) {
                        display_modes.push(mode);
                    }
                }
            }

            CFRelease(modes_array as CFTypeRef);

            if display_modes.is_empty() {
                return Err(anyhow!("No display modes found"));
            }

            // Sort by resolution, then by refresh rate
            display_modes.sort_by(
                |a, b| match (a.width * a.height).cmp(&(b.width * b.height)) {
                    std::cmp::Ordering::Equal => {
                        a.refresh_rate.partial_cmp(&b.refresh_rate).unwrap()
                    }
                    other => other,
                },
            );

            Ok(display_modes)
        }
    }

    pub async fn set_display_mode(&self, mode: &DisplayMode) -> Result<()> {
        unsafe {
            let modes_array = CGDisplayCopyAllDisplayModes(self.display_id, std::ptr::null());
            if modes_array.is_null() {
                return Err(anyhow!("Failed to get display modes"));
            }

            let count = CFArray::wrap_under_create_rule(modes_array).len();
            let mut target_mode = None;

            for i in 0..count {
                let mode_ref = CFArray::wrap_under_create_rule(modes_array).get(i);
                if mode_ref.is_null() {
                    continue;
                }

                let mode_ref = mode_ref as CGDisplayModeRef;
                let width = CGDisplayModeGetWidth(mode_ref) as u32;
                let height = CGDisplayModeGetHeight(mode_ref) as u32;
                let refresh_rate = CGDisplayModeGetRefreshRate(mode_ref);

                if width == mode.width
                    && height == mode.height
                    && (refresh_rate - mode.refresh_rate).abs() < 0.1
                {
                    target_mode = Some(mode_ref);
                    break;
                }
            }

            let target_mode = match target_mode {
                Some(mode_ref) => mode_ref,
                None => {
                    CFRelease(modes_array as CFTypeRef);
                    return Err(anyhow!(
                        "Display mode {}x{}@{}Hz not available",
                        mode.width,
                        mode.height,
                        mode.refresh_rate
                    ));
                }
            };

            let result = CGDisplaySetDisplayMode(self.display_id, target_mode, std::ptr::null());
            CFRelease(modes_array as CFTypeRef);

            if result != 0 {
                return Err(anyhow!(
                    "Failed to set display mode. Core Graphics error: {}",
                    result
                ));
            }
        }

        Ok(())
    }

    pub async fn get_current_display_mode(&self) -> Result<DisplayMode> {
        unsafe {
            use core_graphics::display::{CGDisplayCopyDisplayMode, CGDisplayModeRelease};

            let current_mode = CGDisplayCopyDisplayMode(self.display_id);
            if current_mode.is_null() {
                return Err(anyhow!("Failed to get current display mode"));
            }

            let width = CGDisplayModeGetWidth(current_mode) as u32;
            let height = CGDisplayModeGetHeight(current_mode) as u32;
            let refresh_rate = CGDisplayModeGetRefreshRate(current_mode);

            CGDisplayModeRelease(current_mode);

            Ok(DisplayMode {
                width,
                height,
                refresh_rate,
            })
        }
    }
}
