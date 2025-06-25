use clap::Parser;
use anyhow::Result;

mod cli;
mod display;
mod parser;
mod profile;
mod platform;

use cli::{Args, ParsedArgs};
use display::{DisplayManager, DisplaySpec};
use profile::ProfileManager;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse().to_parsed_args();
    
    let display_manager = DisplayManager::new()?;
    let mut profile_manager = ProfileManager::new()?;

    match args {
        ParsedArgs::Switch { spec, exact } => {
            handle_switch(&display_manager, spec, exact).await?;
        }
        ParsedArgs::List { spec, json } => {
            handle_list(&display_manager, spec, json).await?;
        }
        ParsedArgs::CreateProfile { name, spec } => {
            handle_create_profile(&mut profile_manager, name, spec)?;
        }
        ParsedArgs::Profile { name } => {
            handle_profile(&display_manager, &profile_manager, name).await?;
        }
        ParsedArgs::ListProfiles => {
            handle_list_profiles(&profile_manager)?;
        }
    }

    Ok(())
}

async fn handle_switch(
    display_manager: &DisplayManager,
    specs: Vec<String>,
    exact: bool,
) -> Result<()> {
    let parsed_specs: Result<Vec<DisplaySpec>, _> = specs
        .iter()
        .map(|s| parser::parse_display_spec(s))
        .collect();
    let parsed_specs = parsed_specs?;

    for spec in parsed_specs {
        match display_manager.switch_display(&spec, exact).await {
            Ok(actual_mode) => {
                println!("Successfully switched to display specification: {} (requested: {})", actual_mode, spec);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to switch to {}: {}", spec, e);
                continue;
            }
        }
    }

    anyhow::bail!("No suitable display specification could be applied");
}

async fn handle_list(
    display_manager: &DisplayManager, 
    filter_spec: Option<String>,
    json: bool,
) -> Result<()> {
    let available_modes = display_manager.list_available_modes().await?;
    
    let filtered_modes = if let Some(filter) = filter_spec {
        let filter_spec = parser::parse_display_spec(&filter)?;
        available_modes
            .into_iter()
            .filter(|mode| mode.matches_filter(&filter_spec))
            .collect()
    } else {
        available_modes
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered_modes)?);
    } else {
        for mode in filtered_modes {
            println!("{}", mode);
        }
    }

    Ok(())
}

fn handle_create_profile(
    profile_manager: &mut ProfileManager,
    name: String,
    specs: Vec<String>,
) -> Result<()> {
    let parsed_specs: Result<Vec<DisplaySpec>, _> = specs
        .iter()
        .map(|s| parser::parse_display_spec(s))
        .collect();
    let parsed_specs = parsed_specs?;

    profile_manager.create_profile(name.clone(), parsed_specs)?;
    println!("Created profile: {}", name);
    Ok(())
}

async fn handle_profile(
    display_manager: &DisplayManager,
    profile_manager: &ProfileManager,
    name: String,
) -> Result<()> {
    let specs = profile_manager.get_profile(&name)?;
    
    for spec in specs {
        match display_manager.switch_display(&spec, false).await {
            Ok(actual_mode) => {
                println!("Successfully switched to profile '{}' with specification: {} (requested: {})", name, actual_mode, spec);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to switch to {}: {}", spec, e);
                continue;
            }
        }
    }

    anyhow::bail!("No suitable display specification in profile '{}' could be applied", name);
}

fn handle_list_profiles(profile_manager: &ProfileManager) -> Result<()> {
    let profiles = profile_manager.list_profiles()?;
    
    if profiles.is_empty() {
        println!("No profiles found.");
        return Ok(());
    }

    for (name, specs) in profiles {
        println!("Profile: {}", name);
        for spec in specs {
            println!("  - {}", spec);
        }
        println!();
    }

    Ok(())
} 