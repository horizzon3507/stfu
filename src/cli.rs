use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(
    name = "stfu",
    version,
    about = "Kill switch for all microphones (PipeWire/WirePlumber)",
    long_about = "Hardware-like kill switch that disables ALSA, Bluetooth (BlueZ) and virtual audio capture inputs via WirePlumber drop-in configuration rules."
)]
pub struct Cli {
    #[arg(long, global = true, help = "Output formatted JSON")]
    pub json: bool,

    #[arg(short, long, global = true, help = "Suppress non-error output")]
    pub quiet: bool,

    #[arg(short, long, global = true, help = "Verbose diagnostic logging")]
    pub verbose: bool,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
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
    /// Toggle microphone state
    Toggle {
        /// Don't restart wireplumber (applies on next restart)
        #[arg(long)]
        no_restart: bool,
    },
    /// Print current mic state: "on" or "off"
    Status,
    /// Restart the WirePlumber audio daemon
    Restart,
    /// List detected audio capture sources
    List,
    /// Generate shell completions for your current shell
    Completions {
        #[arg(value_enum)]
        shell: ShellType,
    },
    /// Generate manpage (stfu.1)
    Manpage,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Serialize)]
pub struct StatusOutput {
    pub killswitch_active: bool,
    pub state: String,
    pub config_path: String,
    pub daemon_running: bool,
}

#[derive(Debug, Serialize)]
pub struct CommandOutput {
    pub success: bool,
    pub state: String,
    pub message: String,
}
