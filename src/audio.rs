use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::error::{StfuError, StfuResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioSource {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub source_type: String,
    pub active: bool,
}

pub fn list_audio_sources() -> StfuResult<Vec<AudioSource>> {
    // Try wpctl status first
    let output = Command::new("wpctl")
        .arg("status")
        .output()
        .map_err(|e| StfuError::DeviceParse(format!("Failed to execute wpctl: {e}")))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    parse_sources_from_wpctl(&text)
}

pub fn parse_sources_from_wpctl(text: &str) -> StfuResult<Vec<AudioSource>> {
    let mut sources = Vec::new();
    let mut in_sources_section = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("├─ Sources:") || trimmed.starts_with("Sources:") || trimmed.starts_with("Audio") && trimmed.contains("Sources:") {
            in_sources_section = true;
            continue;
        }

        if in_sources_section {
            if trimmed.starts_with("├─ Filters:")
                || trimmed.starts_with("├─ Streams:")
                || trimmed.starts_with("└─ Streams:")
                || trimmed.starts_with("Video")
                || trimmed.starts_with("Settings")
            {
                break;
            }

            if trimmed.is_empty() {
                continue;
            }

            // Lines like:
            // │  *   56. USB PnP Sound Device Estéreo analógico [vol: 1.00]
            // │      50. Ryzen HD Audio Controller
            let clean = trimmed.trim_start_matches(|c| c == '│' || c == '├' || c == '└' || c == '─' || c == ' ').trim();
            let is_default = clean.starts_with('*');
            let clean_no_default = clean.trim_start_matches('*').trim();

            if let Some((id_part, rest)) = clean_no_default.split_once('.') {
                if let Ok(id) = id_part.trim().parse::<u32>() {
                    let name_and_props = rest.trim();
                    let desc = if let Some((name_part, _vol)) = name_and_props.split_once('[') {
                        name_part.trim()
                    } else {
                        name_and_props
                    };

                    let source_type = if desc.to_lowercase().contains("bluez") || desc.to_lowercase().contains("bluetooth") {
                        "bluetooth"
                    } else if desc.to_lowercase().contains("usb") {
                        "usb"
                    } else {
                        "alsa"
                    };

                    sources.push(AudioSource {
                        id,
                        name: format!("source-{id}"),
                        description: desc.to_string(),
                        source_type: source_type.to_string(),
                        active: is_default,
                    });
                }
            }
        }
    }

    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sources() {
        let sample = r#"
Audio
 ├─ Devices:
 │      59. USB PnP Sound Device                [alsa]
 ├─ Sinks:
 │  *   56. USB PnP Sound Device Estéreo analógico [vol: 1.00]
 ├─ Sources:
 │  *   57. USB PnP Sound Device Mono [vol: 0.85]
 │      60. Built-in Microphone [vol: 1.00]
 ├─ Filters:
"#;
        let sources = parse_sources_from_wpctl(sample).unwrap();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].id, 57);
        assert_eq!(sources[0].description, "USB PnP Sound Device Mono");
        assert_eq!(sources[0].source_type, "usb");
        assert!(sources[0].active);
        assert_eq!(sources[1].id, 60);
        assert_eq!(sources[1].description, "Built-in Microphone");
        assert_eq!(sources[1].source_type, "alsa");
        assert!(!sources[1].active);
    }
}
