use std::process::Command;

use crate::error::{StfuError, StfuResult};

pub fn restart_wireplumber() -> StfuResult<()> {
    let status = Command::new("systemctl")
        .args(["--user", "restart", "wireplumber"])
        .status()
        .map_err(|e| StfuError::SystemctlExec(e.to_string()))?;

    if status.success() {
        Ok(())
    } else {
        Err(StfuError::SystemctlStatus(format!("systemctl exited with {status}")))
    }
}

pub fn is_wireplumber_active() -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "wireplumber"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
