mod audio;
mod cli;
mod config;
mod daemon;
mod error;

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::fs;
use std::io;
use std::process::ExitCode;

use cli::{Cli, Cmd, CommandOutput, ShellType, StatusOutput};
use error::StfuResult;

fn run() -> StfuResult<bool> {
    let cli = Cli::parse();
    let conf_path = config::conf_file()?;
    let dir = config::dropin_dir()?;

    match cli.cmd {
        Cmd::On { no_restart } => {
            if conf_path.exists() {
                if let Err(e) = fs::remove_file(&conf_path) {
                    if e.kind() != io::ErrorKind::NotFound {
                        return Err(error::StfuError::Io {
                            path: conf_path.display().to_string(),
                            source: e,
                        });
                    }
                }
            }
            let _ = config::clean_stale_files(&dir);

            if !no_restart {
                daemon::restart_wireplumber()?;
            }

            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&CommandOutput {
                        success: true,
                        state: "on".into(),
                        message: "Microphones enabled".into(),
                    })?
                );
            } else if !cli.quiet {
                println!("on");
            }
        }
        Cmd::Off { no_restart } => {
            config::write_dropin_atomic(&dir, config::CONF_NAME, config::DROPIN_CONTENT)?;
            let _ = config::clean_stale_files(&dir);

            if !no_restart {
                daemon::restart_wireplumber()?;
            }

            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&CommandOutput {
                        success: true,
                        state: "off".into(),
                        message: "Microphones disabled (STFU engaged)".into(),
                    })?
                );
            } else if !cli.quiet {
                println!("off");
            }
        }
        Cmd::Toggle { no_restart } => {
            let active = config::is_killswitch_active()?;
            let new_state = if active {
                // Currently muted -> enable
                if conf_path.exists() {
                    let _ = fs::remove_file(&conf_path);
                }
                "on"
            } else {
                // Currently unmuted -> disable
                config::write_dropin_atomic(&dir, config::CONF_NAME, config::DROPIN_CONTENT)?;
                "off"
            };

            let _ = config::clean_stale_files(&dir);

            if !no_restart {
                daemon::restart_wireplumber()?;
            }

            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&CommandOutput {
                        success: true,
                        state: new_state.into(),
                        message: format!("Microphones toggled to {new_state}"),
                    })?
                );
            } else if !cli.quiet {
                println!("{new_state}");
            }
        }
        Cmd::Status => {
            let active = config::is_killswitch_active()?;
            let state = if active { "off" } else { "on" };
            let daemon_running = daemon::is_wireplumber_active();

            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&StatusOutput {
                        killswitch_active: active,
                        state: state.into(),
                        config_path: conf_path.display().to_string(),
                        daemon_running,
                    })?
                );
            } else if !cli.quiet {
                println!("{state}");
            }
        }
        Cmd::Restart => {
            daemon::restart_wireplumber()?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&CommandOutput {
                        success: true,
                        state: if config::is_killswitch_active()? { "off".into() } else { "on".into() },
                        message: "WirePlumber daemon restarted".into(),
                    })?
                );
            } else if !cli.quiet {
                println!("WirePlumber restarted successfully");
            }
        }
        Cmd::List => {
            let sources = audio::list_audio_sources()?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&sources)?);
            } else if !cli.quiet {
                if sources.is_empty() {
                    println!("No active audio capture sources found (or microphones killed).");
                } else {
                    println!("Detected audio capture sources:");
                    for s in sources {
                        let active_marker = if s.active { "*" } else { " " };
                        println!(" {} [{}] #{} - {} ({})", active_marker, s.source_type, s.id, s.description, s.name);
                    }
                }
            }
        }
        Cmd::Completions { shell } => {
            let mut cmd = Cli::command();
            let bin_name = "stfu";
            match shell {
                ShellType::Bash => generate(Shell::Bash, &mut cmd, bin_name, &mut io::stdout()),
                ShellType::Zsh => generate(Shell::Zsh, &mut cmd, bin_name, &mut io::stdout()),
                ShellType::Fish => generate(Shell::Fish, &mut cmd, bin_name, &mut io::stdout()),
            }
        }
        Cmd::Manpage => {
            let cmd = Cli::command();
            let man = clap_mangen::Man::new(cmd);
            man.render(&mut io::stdout())
                .map_err(|e| error::StfuError::Io {
                    path: "stdout".into(),
                    source: e,
                })?;
        }
    }

    Ok(true)
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("stfu error: {e}");
            ExitCode::FAILURE
        }
    }
}
