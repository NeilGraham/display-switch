use anyhow::{anyhow, Result};
use regex::Regex;

use crate::display::DisplaySpec;

pub fn parse_display_spec(spec: &str) -> Result<DisplaySpec> {
    let spec = spec.trim().to_lowercase();
    
    // Split by @ to separate resolution/aspect from refresh rate
    let parts: Vec<&str> = spec.split('@').collect();
    let resolution_part = parts[0];
    let refresh_rate = if parts.len() > 1 {
        Some(parse_refresh_rate(parts[1])?)
    } else {
        None
    };

    // Try to parse as resolution first, then as aspect ratio
    if let Ok((width, height)) = parse_resolution(resolution_part) {
        return Ok(DisplaySpec {
            width: Some(width),
            height: Some(height),
            refresh_rate,
            aspect_ratio: None,
        });
    }

    if let Ok((w_ratio, h_ratio)) = parse_aspect_ratio(resolution_part) {
        return Ok(DisplaySpec {
            width: None,
            height: None,
            refresh_rate,
            aspect_ratio: Some((w_ratio, h_ratio)),
        });
    }

    Err(anyhow!("Unable to parse display specification: {}", spec))
}

fn parse_resolution(resolution: &str) -> Result<(u32, u32)> {
    // Pattern: {width}x{height} (e.g., "1920x1080", "2560x1440")
    let width_height_regex = Regex::new(r"^(\d+)x(\d+)$").unwrap();
    if let Some(captures) = width_height_regex.captures(resolution) {
        let width = captures[1].parse::<u32>()?;
        let height = captures[2].parse::<u32>()?;
        return Ok((width, height));
    }

    // Pattern: {height}p (e.g., "1080p", "720p", "4320p")
    let height_p_regex = Regex::new(r"^(\d+)p$").unwrap();
    if let Some(captures) = height_p_regex.captures(resolution) {
        let height = captures[1].parse::<u32>()?;
        let width = calculate_width_from_height(height);
        return Ok((width, height));
    }

    // Pattern: {int}k (e.g., "4k", "2k", "8k")
    let k_regex = Regex::new(r"^(\d+)k$").unwrap();
    if let Some(captures) = k_regex.captures(resolution) {
        let k = captures[1].parse::<u32>()?;
        return match k {
            2 => Ok((2048, 1080)),    // 2K DCI
            4 => Ok((3840, 2160)),    // 4K UHD
            8 => Ok((7680, 4320)),    // 8K UHD
            _ => Err(anyhow!("Unsupported K resolution: {}k", k)),
        };
    }

    // Pattern: {height} (interlaced, e.g., "1080i", "720i")
    let height_i_regex = Regex::new(r"^(\d+)i?$").unwrap();
    if let Some(captures) = height_i_regex.captures(resolution) {
        let height = captures[1].parse::<u32>()?;
        let width = calculate_width_from_height(height);
        return Ok((width, height));
    }

    Err(anyhow!("Unable to parse resolution: {}", resolution))
}

fn parse_aspect_ratio(aspect: &str) -> Result<(u32, u32)> {
    // Pattern: {width}:{height} (e.g., "16:9", "4:3", "21:9")
    let aspect_regex = Regex::new(r"^(\d+):(\d+)$").unwrap();
    if let Some(captures) = aspect_regex.captures(aspect) {
        let width_ratio = captures[1].parse::<u32>()?;
        let height_ratio = captures[2].parse::<u32>()?;
        return Ok((width_ratio, height_ratio));
    }

    Err(anyhow!("Unable to parse aspect ratio: {}", aspect))
}

fn parse_refresh_rate(rate: &str) -> Result<f64> {
    // Pattern: {decimal}hz (e.g., "60hz", "144hz", "240hz")
    let hz_regex = Regex::new(r"^([0-9]*\.?[0-9]+)hz$").unwrap();
    if let Some(captures) = hz_regex.captures(rate) {
        return Ok(captures[1].parse::<f64>()?);
    }

    // Pattern: {decimal}fps (e.g., "60fps", "59.94fps", "120fps")
    let fps_regex = Regex::new(r"^([0-9]*\.?[0-9]+)fps$").unwrap();
    if let Some(captures) = fps_regex.captures(rate) {
        return Ok(captures[1].parse::<f64>()?);
    }

    Err(anyhow!("Unable to parse refresh rate: {}", rate))
}

fn calculate_width_from_height(height: u32) -> u32 {
    // Common aspect ratios and their widths for given heights
    match height {
        480 => 640,    // 4:3
        576 => 768,    // 4:3 PAL
        720 => 1280,   // 16:9
        1080 => 1920,  // 16:9
        1440 => 2560,  // 16:9
        2160 => 3840,  // 16:9 4K
        4320 => 7680,  // 16:9 8K
        // Default to 16:9 aspect ratio
        _ => (height * 16) / 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resolution() {
        assert_eq!(parse_resolution("1920x1080").unwrap(), (1920, 1080));
        assert_eq!(parse_resolution("2560x1440").unwrap(), (2560, 1440));
        assert_eq!(parse_resolution("1080p").unwrap(), (1920, 1080));
        assert_eq!(parse_resolution("4k").unwrap(), (3840, 2160));
        assert_eq!(parse_resolution("1080i").unwrap(), (1920, 1080));
    }

    #[test]
    fn test_parse_aspect_ratio() {
        assert_eq!(parse_aspect_ratio("16:9").unwrap(), (16, 9));
        assert_eq!(parse_aspect_ratio("4:3").unwrap(), (4, 3));
        assert_eq!(parse_aspect_ratio("21:9").unwrap(), (21, 9));
    }

    #[test]
    fn test_parse_refresh_rate() {
        assert_eq!(parse_refresh_rate("60hz").unwrap(), 60.0);
        assert_eq!(parse_refresh_rate("144hz").unwrap(), 144.0);
        assert_eq!(parse_refresh_rate("59.94fps").unwrap(), 59.94);
        assert_eq!(parse_refresh_rate("120fps").unwrap(), 120.0);
    }

    #[test]
    fn test_parse_display_spec() {
        let spec = parse_display_spec("1920x1080@60hz").unwrap();
        assert_eq!(spec.width, Some(1920));
        assert_eq!(spec.height, Some(1080));
        assert_eq!(spec.refresh_rate, Some(60.0));

        let spec = parse_display_spec("16:9@120fps").unwrap();
        assert_eq!(spec.aspect_ratio, Some((16, 9)));
        assert_eq!(spec.refresh_rate, Some(120.0));

        let spec = parse_display_spec("4k").unwrap();
        assert_eq!(spec.width, Some(3840));
        assert_eq!(spec.height, Some(2160));
        assert_eq!(spec.refresh_rate, None);
    }
} 