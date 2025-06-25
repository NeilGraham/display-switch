use anyhow::{anyhow, Result};
use std::ffi::CStr;
use std::ptr;
use x11::xlib::{Display, XCloseDisplay, XDefaultScreen, XOpenDisplay, XRootWindow};
use x11::xrandr::{
    XRRConfigCurrentConfiguration, XRRConfigRates, XRRConfigSizes,
    XRRFreeScreenConfigInfo, XRRGetScreenInfo, XRRSetScreenConfigAndRate,
    XRRScreenConfiguration, XRRScreenSize,
};

use crate::display::DisplayMode;

pub struct LinuxDisplayManager {
    display: *mut Display,
}

impl LinuxDisplayManager {
    pub fn new() -> Result<Self> {
        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                return Err(anyhow!("Failed to open X11 display"));
            }

            Ok(Self { display })
        }
    }

    pub async fn get_available_modes(&self) -> Result<Vec<DisplayMode>> {
        let mut modes = Vec::new();

        unsafe {
            let screen = XDefaultScreen(self.display);
            let root = XRootWindow(self.display, screen);
            let screen_info = XRRGetScreenInfo(self.display, root);

            if screen_info.is_null() {
                return Err(anyhow!("Failed to get screen info"));
            }

            let mut num_sizes = 0;
            let sizes = XRRConfigSizes(screen_info, &mut num_sizes);

            if sizes.is_null() || num_sizes == 0 {
                XRRFreeScreenConfigInfo(screen_info);
                return Err(anyhow!("No screen sizes available"));
            }

            for i in 0..num_sizes {
                let size = *sizes.offset(i as isize);
                
                let mut num_rates = 0;
                let rates = XRRConfigRates(screen_info, i, &mut num_rates);

                if !rates.is_null() && num_rates > 0 {
                    for j in 0..num_rates {
                        let rate = *rates.offset(j as isize);
                        
                        let mode = DisplayMode {
                            width: size.width as u32,
                            height: size.height as u32,
                            refresh_rate: rate as f64,
                        };

                        // Avoid duplicates
                        if !modes.iter().any(|m| {
                            m.width == mode.width 
                            && m.height == mode.height 
                            && (m.refresh_rate - mode.refresh_rate).abs() < 0.1
                        }) {
                            modes.push(mode);
                        }
                    }
                }
            }

            XRRFreeScreenConfigInfo(screen_info);
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
            let screen = XDefaultScreen(self.display);
            let root = XRootWindow(self.display, screen);
            let screen_info = XRRGetScreenInfo(self.display, root);

            if screen_info.is_null() {
                return Err(anyhow!("Failed to get screen info"));
            }

            let mut num_sizes = 0;
            let sizes = XRRConfigSizes(screen_info, &mut num_sizes);

            if sizes.is_null() || num_sizes == 0 {
                XRRFreeScreenConfigInfo(screen_info);
                return Err(anyhow!("No screen sizes available"));
            }

            // Find matching size index
            let mut size_index = None;
            for i in 0..num_sizes {
                let size = *sizes.offset(i as isize);
                if size.width as u32 == mode.width && size.height as u32 == mode.height {
                    size_index = Some(i);
                    break;
                }
            }

            let size_index = match size_index {
                Some(idx) => idx,
                None => {
                    XRRFreeScreenConfigInfo(screen_info);
                    return Err(anyhow!(
                        "Resolution {}x{} not available",
                        mode.width,
                        mode.height
                    ));
                }
            };

            // Find matching refresh rate
            let mut num_rates = 0;
            let rates = XRRConfigRates(screen_info, size_index, &mut num_rates);
            let mut rate_index = None;

            if !rates.is_null() && num_rates > 0 {
                for j in 0..num_rates {
                    let rate = *rates.offset(j as isize);
                    if (rate as f64 - mode.refresh_rate).abs() < 0.1 {
                        rate_index = Some(rate);
                        break;
                    }
                }
            }

            let rate = match rate_index {
                Some(r) => r,
                None => {
                    XRRFreeScreenConfigInfo(screen_info);
                    return Err(anyhow!(
                        "Refresh rate {}Hz not available for {}x{}",
                        mode.refresh_rate,
                        mode.width,
                        mode.height
                    ));
                }
            };

            // Apply the configuration
            let result = XRRSetScreenConfigAndRate(
                self.display,
                screen_info,
                root,
                size_index,
                0, // rotation
                rate,
                0, // timestamp
            );

            XRRFreeScreenConfigInfo(screen_info);

            if result != 0 {
                return Err(anyhow!("Failed to set display mode. XRandR error: {}", result));
            }
        }

        Ok(())
    }
}

impl Drop for LinuxDisplayManager {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                XCloseDisplay(self.display);
            }
        }
    }
} 