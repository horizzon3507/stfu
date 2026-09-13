use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

const CONF_NAME: &str = "50-disable-mics.conf";
const EXT_UUID: &str = "stfu@firefly";
const EXT_DIR: &str = "gnome-shell/extensions";

const DROPIN: &str = r#"# STFU: disables all ALSA capture nodes (PipeWire/WirePlumber)
monitor.alsa.rules = [
  {
    matches = [
      {
        node.name = "~alsa_input.*"
      }
    ]
    actions = {
      update-props = {
        node.disabled = true
      }
    }
  }
]
"#;

const EXT_METADATA: &str = include_str!("../gnome-extension/metadata.json");
const EXT_JS: &str = include_str!("../gnome-extension/extension.js");

#[derive(Parser)]
#[command(name = "stfu", version, about = "Kill switch for all microphones (PipeWire/WirePlumber)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Enable all microphones
    On {
        /// Don't restart wireplumber (applies on next restart)
        #[arg(long)]
        no_restart: bool,
    },
    /// Disable all microphones (STFU engaged)
    Off {
        /// Don't restart wireplumber (applies on next restart)
        #[arg(long)]
        no_restart: bool,
    },
    /// Toggle microphone state, prints the new state
    Toggle {
        /// Don't restart wireplumber (applies on next restart)
        #[arg(long)]
        no_restart: bool,
    },
    /// Print current mic state: "on" or "off"
    Status,
    /// Install the GNOME Shell extension (quick settings toggle)
    InstallExtension,
}

fn dropin_dir() -> PathBuf {
    let cfg = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").expect("HOME not set")).join(".config"));
    cfg.join("wireplumber").join("wireplumber.conf.d")
}

fn conf_file() -> PathBuf {
    dropin_dir().join(CONF_NAME)
}

fn disabled_file() -> PathBuf {
    dropin_dir().join(format!("{CONF_NAME}.disabled"))
}

/// mics are "off" (muted) when the drop-in is active
fn mics_off() -> bool {
    conf_file().exists()
}

fn state_str() -> &'static str {
    if mics_off() { "off" } else { "on" }
}

fn restart_wireplumber() -> Result<(), String> {
    let status = Command::new("systemctl")
        .args(["--user", "restart", "wireplumber"])
        .status()
        .map_err(|e| format!("failed to run systemctl: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("systemctl exited with {status}"))
    }
}

fn set_mics(on: bool, no_restart: bool) -> Result<(), String> {
    fs::create_dir_all(dropin_dir()).map_err(|e| format!("mkdir failed: {e}"))?;
    let _ = fs::remove_file(disabled_file()); // clean up stale toggle state
    if on {
        match fs::remove_file(conf_file()) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("failed to remove {}: {e}", conf_file().display())),
        }
    } else {
        fs::write(conf_file(), DROPIN).map_err(|e| format!("failed to write {}: {e}", conf_file().display()))?;
    }
    if !no_restart {
        restart_wireplumber()?;
    }
    Ok(())
}

fn install_extension() -> Result<(), String> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").expect("HOME not set")).join(".local/share"));
    let dir = data.join(EXT_DIR).join(EXT_UUID);
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    fs::write(dir.join("metadata.json"), EXT_METADATA).map_err(|e| format!("write metadata.json: {e}"))?;
    fs::write(dir.join("extension.js"), EXT_JS).map_err(|e| format!("write extension.js: {e}"))?;
    println!("installed to {}", dir.display());

    // gnome-shell only scans extension dirs at startup; on Wayland that means relogin
    let enabled = Command::new("gnome-extensions")
        .args(["enable", EXT_UUID])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if enabled {
        println!("extension enabled");
    } else {
        println!("now log out/in (or Alt+F2 r on X11), then: gnome-extensions enable {EXT_UUID}");
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::On { no_restart } => set_mics(true, no_restart).map(|()| println!("{}", state_str())),
        Cmd::Off { no_restart } => set_mics(false, no_restart).map(|()| println!("{}", state_str())),
        Cmd::Toggle { no_restart } => {
            let target = mics_off(); // mics off -> turn on
            set_mics(target, no_restart).map(|()| println!("{}", state_str()))
        }
        Cmd::Status => Ok(println!("{}", state_str())),
        Cmd::InstallExtension => install_extension(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("stfu: {e}");
            ExitCode::FAILURE
        }
    }
}
