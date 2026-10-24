# STFU (Shut The F*** Up) 🤫

Hardware-like kill switch for all audio capture devices (PipeWire / WirePlumber) on Linux.

Instead of software muting audio applications or sliders (which software can unmute or bypass), `stfu` disables the hardware and software input nodes directly at the WirePlumber policy level (`node.disabled = true`).

## Features

- **Wide Device Coverage**: Disables ALSA hardware inputs (`~alsa_input.*`), Bluetooth headset/AirPods inputs (`~bluez_input.*`), and virtual/v4l2 audio capture sources.
- **Atomic Operations**: Safe drop-in file writes using POSIX atomic renames (`NamedTempFile`), preventing corrupted or half-written configurations.
- **Self-Healing Cleanup**: Automatically cleans up stale temporary and `.disabled` backup files.
- **Machine & Scripting Friendly**: Full `--json`, `--quiet`, and `--verbose` support for status and toggle automation.
- **Completions & Manpages**: Generates shell autocompletions (Bash, Zsh, Fish) and standard manual pages (`stfu.1`).

## Installation

### Arch Linux (AUR)

`stfu` is available in the Arch User Repository (AUR):

```bash
paru -S stfu
# or
yay -S stfu
```

### From Source (Cargo)

```bash
git clone https://github.com/fireflylabss/stfu.git
cd stfu
cargo install --path .
```

## Usage

```bash
# Disable all microphones (Kill switch engaged)
stfu off

# Enable all microphones
stfu on

# Toggle microphone state
stfu toggle

# Check state (returns 'on' or 'off')
stfu status

# Check state with detailed JSON
stfu status --json

# List detected audio capture devices in PipeWire
stfu list

# Restart WirePlumber audio daemon
stfu restart

# Generate shell completions
stfu completions bash > ~/.local/share/bash-completion/completions/stfu
stfu completions zsh > ~/.zfunc/_stfu
stfu completions fish > ~/.config/fish/completions/stfu.fish
```

## How It Works

`stfu` generates an atomic drop-in configuration file at `$XDG_CONFIG_HOME/wireplumber/wireplumber.conf.d/50-disable-mics.conf`:

```spa-json
monitor.alsa.rules = [
  {
    matches = [ { node.name = "~alsa_input.*" } ]
    actions = { update-props = { node.disabled = true } }
  }
]
monitor.bluez.rules = [
  {
    matches = [ { node.name = "~bluez_input.*" } ]
    actions = { update-props = { node.disabled = true } }
  }
]
wireplumber.rules = [
  {
    matches = [ { media.class = "Audio/Source" }, { media.class = "Audio/Source/Virtual" } ]
    actions = { update-props = { node.disabled = true } }
  }
]
```

When enabled, WirePlumber stops creating or exposing audio capture nodes to apps, effectively disabling input at the daemon level.

## License

[MIT](LICENSE)
