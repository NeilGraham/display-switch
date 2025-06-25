# Display Switch

A cross-platform (Windows, Linux, MacOS) Rust CLI tool for switching and listing display specifications. Useful for services like Sunshine and Moonlight to switch a server to its client's display specification.

## Installation

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
