# Display Switch

A cross-platform (Windows, Linux, MacOS) Rust CLI tool for switching and listing display specifications. Useful for services like Sunshine and Moonlight to switch a server to its client's display specification.

## Installation

### Download Pre-built Binaries

Pre-built binaries are available for Windows on the [Releases page](../../releases).

- **Windows**: `display-switch-windows-x86_64.exe`

<!-- Other platforms are commented out in CI:
- **Linux**: `display-switch-linux-x86_64` 
- **macOS Intel**: `display-switch-macos-x86_64`
- **macOS Apple Silicon**: `display-switch-macos-aarch64`
-->

### Building from source

```bash
# Clone the repository
git clone https://github.com/yourusername/display-switch.git
cd display-switch

# Build the project
cargo build --release

# The binary will be available at target/release/display-switch
```

### Prerequisites

- **Windows**: No additional dependencies required
- **Linux**: X11 development libraries (`libx11-dev` and `libxrandr-dev` on Ubuntu/Debian)
- **macOS**: No additional dependencies required

## Development

### Creating Releases

To create a new release with pre-built binaries:

1. Create and push a new tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. GitHub Actions will automatically:
   - Build binaries for Windows
   - Create a new GitHub release
   - Attach the binaries to the release

### GitHub Actions

This project uses GitHub Actions for CI/CD:

- **CI Workflow** (`.github/workflows/ci.yml`): Runs on every push/PR to test builds and run linting (Windows only)
- **Build Workflow** (`.github/workflows/build.yml`): Creates release binaries when tags are pushed (Windows only)

> **Note**: Linux and macOS builds are currently commented out in the workflows. To enable them, uncomment the relevant sections in the workflow files.

## CLI Usage

#### Switch to a display specification:

```bash
display-switch --spec 2560x1440@120hz 1080p@120hz 16:9@60hz --exact
```
- If multiple specifications (`-s`/`--spec`) are specified, it will attempt the first one that works in the order they were given.
- `--exact` being specified forces an exact match, otherwise the closest match will be used.
- Available formats for resolution:
    - `{width}x{height}` (e.g. `1920x1080`, `7680x2160`)
    - `{height}p` (e.g. `1080p`, `2160p`, `480p`)
    - `{int}k` (e.g. `4k`, `2k`, `8k`)
    - `{height}` (e.g. `1080i`)
- Available formats for refresh rate:
    - `{decimal}hz` (e.g. `240hz`, `60hz`, `30hz`)
    - `{decimal}fps` (e.g. `59.94fps`, `120fps`, `60fps`)
- Format formats for aspect ratio:
    - `{width}:{height}` (e.g. `16:9`, `4:3`, `16:10`)
- Refresh rate can be specified with either resolution or aspect ratio using an `@` separator (e.g. `1080p@240hz`, `16:9@60fps`)

#### List available formats that match the specification filter:

```bash
display-switch --list --json
```
- Lists all available display specifications in json format.

```bash
display-switch --list --spec 16:9@240hz
```
- List all available display modes matching 16:9 aspect ratio and 240Hz refresh rate.

#### Create a named profile:

```bash
display-switch --create-profile "Recording" --spec 4k@60fps
```
- Creates a new profile with name `Recording` and matching specification of 4K at 60Hz refresh rate.
- A sequence of specifications can be specified for fallbacks.

#### Switch to the newly created profile:

```bash
display-switch --profile "Recording"
```

#### List all available profiles:

```bash
display-switch --list-profiles
```
