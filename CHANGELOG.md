# Changelog

We follow [Semantic Versioning](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/).

<details>
<summary>To see more about versioning, expand this.</summary>

Every version string starts with `v` (required), e.g. `v0.2.0`, `v0.1.0`.

Here the installable surface is **CLI**. Other apps swap in their own names the same way — whatever you actually ship.

| Part | What you install | Example |
| --- | --- | --- |
| **CLI** | `stfu` in the terminal | `v0.2.0` |

Each release heading is the version and date (`## v0.2.0 · 24/10/2026`); under it, a short summary ends with a plain sentence like: “This version was made for the CLI on 24/10/2026 (v0.2.0).”

</details>

## v0.2.0 · 24/10/2026

Complete CLI overhaul, atomic safety improvements, broader audio device support, AUR packaging, and removal of the GNOME extension. This version was made for the CLI on 24/10/2026 (v0.2.0).

- Complete CLI architecture rewrite in modular Rust (`error`, `config`, `daemon`, `audio`, `cli`).
- Broader hardware support: WirePlumber drop-in now disables ALSA (`~alsa_input.*`), Bluetooth (`~bluez_input.*`), and virtual capture inputs (`Audio/Source`).
- Atomic configuration writes using `NamedTempFile` and atomic rename to eliminate race conditions and partial file writes.
- Automatic cleanup of stale `.disabled*` and `.tmp*` files left in the WirePlumber directory.
- Structured output support with `--json`, `--quiet`, and `--verbose` flags.
- New subcommands: `list` (inspects audio capture devices) and `restart` (restarts WirePlumber daemon directly).
- Built-in shell completions generation (Bash, Zsh, Fish) and standard manpage (`stfu.1`) generation via `build.rs`.
- Removed GNOME Shell extension in favor of a lean, reliable CLI.
- Added Arch Linux AUR packaging pipeline (`PKGBUILD`, `.SRCINFO`, `scripts/update-aur.sh`, and GitHub Actions release workflow).
- Various other small tweaks

## v0.1.0m-beta · 13/09/2026

Initial release introducing the Rust CLI and the GNOME Shell quick settings toggle for PipeWire/WirePlumber mic kill switch. This version was made for both GNOME and CLI with a beta release channel on 13/09/2026 (v0.1.0m-beta).

### CLI

- `stfu` binary with subcommands: `on`, `off`, `toggle`, `status`, and `install-extension`.
- Direct WirePlumber configuration management (`50-disable-mics.conf`) disabling all ALSA capture nodes (`~alsa_input.*`).
- Automatic or optional deferred WirePlumber daemon restarts (`--no-restart`).
- Built-in installer for the GNOME Shell extension.
- Various other small tweaks

### GNOME

- Quick settings toggle button for quick microphone muting and unmuting.
- System panel status indicator icon showing when microphones are muted.
- Real-time directory monitoring of WirePlumber configs to synchronize state changes made externally.
- Various other small tweaks
