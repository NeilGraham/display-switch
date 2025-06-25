use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::platform::PlatformDisplayManager;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplaySpec {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub refresh_rate: Option<f64>,
    pub aspect_ratio: Option<(u32, u32)>, // (width_ratio, height_ratio)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: f64,
}

pub struct DisplayManager {
    platform_manager: PlatformDisplayManager,
}

impl DisplaySpec {
    pub fn matches_filter(&self, filter: &DisplaySpec) -> bool {
        // Check width and height
        if let (Some(filter_width), Some(filter_height)) = (filter.width, filter.height) {
            if let (Some(width), Some(height)) = (self.width, self.height) {
                if width != filter_width || height != filter_height {
                    return false;
                }
            }
        }

        // Check aspect ratio
        if let Some((filter_w_ratio, filter_h_ratio)) = filter.aspect_ratio {
            if let (Some(width), Some(height)) = (self.width, self.height) {
                let gcd = gcd(width, height);
                let actual_w_ratio = width / gcd;
                let actual_h_ratio = height / gcd;
                if actual_w_ratio != filter_w_ratio || actual_h_ratio != filter_h_ratio {
                    return false;
                }
            }
        }

        // Check refresh rate
        if let Some(filter_rate) = filter.refresh_rate {
            if let Some(rate) = self.refresh_rate {
                if (rate - filter_rate).abs() > 0.1 {
                    return false;
                }
            }
        }

        true
    }

    pub fn to_concrete_spec(&self, available_modes: &[DisplayMode]) -> Option<DisplayMode> {
        // If we have concrete width and height, find exact or closest match
        if let (Some(target_width), Some(target_height)) = (self.width, self.height) {
            return self.find_best_mode_for_resolution(available_modes, target_width, target_height);
        }

        // If we have aspect ratio, find modes matching that aspect ratio
        if let Some((w_ratio, h_ratio)) = self.aspect_ratio {
            let matching_modes: Vec<_> = available_modes
                .iter()
                .filter(|mode| {
                    let gcd = gcd(mode.width, mode.height);
                    let actual_w_ratio = mode.width / gcd;
                    let actual_h_ratio = mode.height / gcd;
                    actual_w_ratio == w_ratio && actual_h_ratio == h_ratio
                })
                .cloned()
                .collect();

            if matching_modes.is_empty() {
                return None;
            }

            return self.find_best_mode_by_refresh_rate(&matching_modes);
        }

        None
    }

    fn find_best_mode_for_resolution(
        &self,
        available_modes: &[DisplayMode],
        target_width: u32,
        target_height: u32,
    ) -> Option<DisplayMode> {
        // First, try to find exact resolution match
        let resolution_matches: Vec<_> = available_modes
            .iter()
            .filter(|mode| mode.width == target_width && mode.height == target_height)
            .cloned()
            .collect();

        if !resolution_matches.is_empty() {
            return self.find_best_mode_by_refresh_rate(&resolution_matches);
        }

        // If no exact resolution match, find the closest resolution
        let mut closest_mode = None;
        let mut min_distance = f64::MAX;

        for mode in available_modes {
            let distance = ((mode.width as f64 - target_width as f64).powi(2) 
                + (mode.height as f64 - target_height as f64).powi(2)).sqrt();
            
            if distance < min_distance {
                min_distance = distance;
                closest_mode = Some(mode.clone());
            }
        }

        closest_mode
    }

    fn find_best_mode_by_refresh_rate(&self, modes: &[DisplayMode]) -> Option<DisplayMode> {
        if modes.is_empty() {
            return None;
        }

        if let Some(target_rate) = self.refresh_rate {
            // First try to find exact refresh rate match
            for mode in modes {
                if (mode.refresh_rate - target_rate).abs() < 0.1 {
                    return Some(mode.clone());
                }
            }

            // If no exact match, prefer higher refresh rates first
            // Find all modes with higher refresh rates than target
            let higher_rates: Vec<_> = modes
                .iter()
                .filter(|mode| mode.refresh_rate > target_rate)
                .collect();

            if !higher_rates.is_empty() {
                // Return the lowest higher rate (closest higher rate)
                return higher_rates
                    .iter()
                    .min_by(|a, b| a.refresh_rate.partial_cmp(&b.refresh_rate).unwrap())
                    .map(|&mode| mode.clone());
            }

            // If no higher rates available, find the highest lower rate
            let lower_rates: Vec<_> = modes
                .iter()
                .filter(|mode| mode.refresh_rate < target_rate)
                .collect();

            if !lower_rates.is_empty() {
                return lower_rates
                    .iter()
                    .max_by(|a, b| a.refresh_rate.partial_cmp(&b.refresh_rate).unwrap())
                    .map(|&mode| mode.clone());
            }

            // Fallback - should not happen if modes is not empty
            modes.first().cloned()
        } else {
            // No refresh rate specified, return the mode with the highest refresh rate
            modes.iter().max_by(|a, b| a.refresh_rate.partial_cmp(&b.refresh_rate).unwrap()).cloned()
        }
    }
}

impl DisplayManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            platform_manager: PlatformDisplayManager::new()?,
        })
    }

    pub async fn switch_display(&self, spec: &DisplaySpec, exact: bool) -> Result<DisplayMode> {
        let available_modes = self.platform_manager.get_available_modes().await?;
        
        let target_mode = if exact {
            // For exact match, find a mode that exactly matches the specification
            self.find_exact_match(spec, &available_modes)
        } else {
            // For closest match, use the spec's logic to find the best mode
            spec.to_concrete_spec(&available_modes)
        };

        match target_mode {
            Some(mode) => {
                self.platform_manager.set_display_mode(&mode).await?;
                Ok(mode)
            }
            None => Err(anyhow!("No suitable display mode found for specification: {}", spec)),
        }
    }

    pub async fn list_available_modes(&self) -> Result<Vec<DisplayMode>> {
        self.platform_manager.get_available_modes().await
    }

    pub async fn get_current_display_mode(&self) -> Result<DisplayMode> {
        self.platform_manager.get_current_display_mode().await
    }

    fn find_exact_match(&self, spec: &DisplaySpec, available_modes: &[DisplayMode]) -> Option<DisplayMode> {
        for mode in available_modes {
            let mode_spec = DisplaySpec {
                width: Some(mode.width),
                height: Some(mode.height),
                refresh_rate: Some(mode.refresh_rate),
                aspect_ratio: None,
            };

            if spec.matches_exact(&mode_spec) {
                return Some(mode.clone());
            }
        }
        None
    }
}

impl DisplaySpec {
    fn matches_exact(&self, other: &DisplaySpec) -> bool {
        // Check width and height
        if let (Some(self_width), Some(self_height)) = (self.width, self.height) {
            if let (Some(other_width), Some(other_height)) = (other.width, other.height) {
                if self_width != other_width || self_height != other_height {
                    return false;
                }
            }
        }

        // Check aspect ratio against actual resolution
        if let Some((w_ratio, h_ratio)) = self.aspect_ratio {
            if let (Some(other_width), Some(other_height)) = (other.width, other.height) {
                let gcd = gcd(other_width, other_height);
                let actual_w_ratio = other_width / gcd;
                let actual_h_ratio = other_height / gcd;
                if w_ratio != actual_w_ratio || h_ratio != actual_h_ratio {
                    return false;
                }
            }
        }

        // Check refresh rate
        if let Some(self_rate) = self.refresh_rate {
            if let Some(other_rate) = other.refresh_rate {
                if (self_rate - other_rate).abs() > 0.1 {
                    return false;
                }
            }
        }

        true
    }
}

impl fmt::Display for DisplaySpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if let (Some(width), Some(height)) = (self.width, self.height) {
            parts.push(format!("{}x{}", width, height));
        } else if let Some((w_ratio, h_ratio)) = self.aspect_ratio {
            parts.push(format!("{}:{}", w_ratio, h_ratio));
        }

        if let Some(rate) = self.refresh_rate {
            if parts.is_empty() {
                parts.push(format!("{}hz", rate));
            } else {
                parts[0] = format!("{}@{}hz", parts[0], rate);
            }
        }

        write!(f, "{}", parts.join(" "))
    }
}

impl DisplayMode {
    pub fn matches_filter(&self, filter: &DisplaySpec) -> bool {
        // Check width and height
        if let (Some(filter_width), Some(filter_height)) = (filter.width, filter.height) {
            if self.width != filter_width || self.height != filter_height {
                return false;
            }
        }

        // Check aspect ratio
        if let Some((filter_w_ratio, filter_h_ratio)) = filter.aspect_ratio {
            let gcd = gcd(self.width, self.height);
            let actual_w_ratio = self.width / gcd;
            let actual_h_ratio = self.height / gcd;
            if actual_w_ratio != filter_w_ratio || actual_h_ratio != filter_h_ratio {
                return false;
            }
        }

        // Check refresh rate
        if let Some(filter_rate) = filter.refresh_rate {
            if (self.refresh_rate - filter_rate).abs() > 0.1 {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}@{}hz", self.width, self.height, self.refresh_rate)
    }
}

// Helper function to calculate greatest common divisor
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_spec_matches_filter() {
        let spec = DisplaySpec {
            width: Some(1920),
            height: Some(1080),
            refresh_rate: Some(60.0),
            aspect_ratio: None,
        };

        let filter1 = DisplaySpec {
            width: Some(1920),
            height: Some(1080),
            refresh_rate: None,
            aspect_ratio: None,
        };
        assert!(spec.matches_filter(&filter1));

        let filter2 = DisplaySpec {
            width: None,
            height: None,
            refresh_rate: Some(60.0),
            aspect_ratio: Some((16, 9)),
        };
        assert!(spec.matches_filter(&filter2));
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(1920, 1080), 120);
        assert_eq!(gcd(16, 9), 1);
        assert_eq!(gcd(4, 3), 1);
    }
} 