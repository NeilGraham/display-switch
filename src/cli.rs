use clap::Parser;

#[derive(Parser)]
#[command(name = "display-switch")]
#[command(about = "A cross-platform CLI tool for switching and listing display specifications")]
#[command(version = "0.1.0")]
pub struct Args {
    /// Optional spec or profile name (checks profiles first, then treats as spec)
    #[arg(value_name = "SPEC_OR_PROFILE")]
    pub spec_or_profile: Option<String>,

    /// Display specifications to try (in order of preference)
    #[arg(short, long, value_name = "SPEC", action = clap::ArgAction::Append)]
    pub spec: Vec<String>,

    /// Force exact match instead of closest match
    #[arg(short, long)]
    pub exact: bool,

    /// List available display specifications
    #[arg(short, long)]
    pub list: bool,

    /// Output in JSON format (used with --list)
    #[arg(short, long)]
    pub json: bool,

    /// Create a named profile
    #[arg(long, value_name = "NAME")]
    pub create_profile: Option<String>,

    /// Switch to a named profile
    #[arg(short = 'p', long, value_name = "NAME")]
    pub profile: Option<String>,

    /// List all available profiles
    #[arg(long)]
    pub list_profiles: bool,

    /// Display current display specification
    #[arg(long)]
    pub current: bool,
}

// Convert the flat args structure to the enum used by main
pub enum ParsedArgs {
    Switch { spec: Vec<String>, exact: bool },
    List { spec: Option<String>, json: bool },
    CreateProfile { name: String, spec: Vec<String> },
    Profile { name: String },
    ListProfiles,
    Current { json: bool },
    // New variant for handling the positional argument that could be either
    SpecOrProfile { value: String, exact: bool },
}

impl Args {
    pub fn into_parsed_args(self) -> ParsedArgs {
        if self.current {
            ParsedArgs::Current { json: self.json }
        } else if self.list_profiles {
            ParsedArgs::ListProfiles
        } else if let Some(name) = self.create_profile {
            ParsedArgs::CreateProfile {
                name,
                spec: self.spec,
            }
        } else if let Some(name) = self.profile {
            ParsedArgs::Profile { name }
        } else if self.list {
            ParsedArgs::List {
                spec: self.spec.first().cloned(),
                json: self.json,
            }
        } else if let Some(value) = self.spec_or_profile {
            // If we have a positional argument and no explicit specs, treat it as spec_or_profile
            if self.spec.is_empty() {
                ParsedArgs::SpecOrProfile {
                    value,
                    exact: self.exact,
                }
            } else {
                // If we have both positional and --spec args, combine them
                let mut all_specs = vec![value];
                all_specs.extend(self.spec);
                ParsedArgs::Switch {
                    spec: all_specs,
                    exact: self.exact,
                }
            }
        } else {
            ParsedArgs::Switch {
                spec: self.spec,
                exact: self.exact,
            }
        }
    }
}
