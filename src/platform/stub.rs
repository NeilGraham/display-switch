use anyhow::{anyhow, Result};
use crate::display::DisplayMode;

pub struct StubDisplayManager;

impl StubDisplayManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub async fn get_available_modes(&self) -> Result<Vec<DisplayMode>> {
        // Return some mock display modes for testing purposes
        Ok(vec![
            DisplayMode {
                width: 1920,
                height: 1080,
                refresh_rate: 60.0,
            },
            DisplayMode {
                width: 1920,
                height: 1080,
                refresh_rate: 144.0,
            },
            DisplayMode {
                width: 2560,
                height: 1440,
                refresh_rate: 60.0,
            },
            DisplayMode {
                width: 2560,
                height: 1440,
                refresh_rate: 144.0,
            },
            DisplayMode {
                width: 3840,
                height: 2160,
                refresh_rate: 60.0,
            },
        ])
    }

    pub async fn set_display_mode(&self, mode: &DisplayMode) -> Result<()> {
        // Stub implementation that doesn't actually change the display
        println!(
            "Stub: Would set display mode to {}x{}@{}Hz",
            mode.width, mode.height, mode.refresh_rate
        );
        Err(anyhow!(
            "Display switching not supported on this platform. This is a stub implementation."
        ))
    }

    pub async fn get_current_display_mode(&self) -> Result<DisplayMode> {
        // Return a mock current display mode for testing
        Ok(DisplayMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60.0,
        })
    }
} 