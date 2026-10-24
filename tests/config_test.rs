use std::fs;
use tempfile::tempdir;

#[path = "../src/config.rs"]
mod config;
#[path = "../src/error.rs"]
mod error;

#[test]
fn test_atomic_write_and_cleanup() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path();

    // 1. Write atomic config
    let file_path = config::write_dropin_atomic(dir_path, config::CONF_NAME, config::DROPIN_CONTENT).unwrap();
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("monitor.alsa.rules"));
    assert!(content.contains("monitor.bluez.rules"));
    assert!(content.contains("wireplumber.rules"));

    // 2. Create stale files
    let stale1 = dir_path.join(format!("{}.disabled.12345", config::CONF_NAME));
    let stale2 = dir_path.join(".tmp_stfu_test.conf");
    let safe_file = dir_path.join("60-other.conf");
    fs::write(&stale1, "stale").unwrap();
    fs::write(&stale2, "stale").unwrap();
    fs::write(&safe_file, "keep").unwrap();

    let removed = config::clean_stale_files(dir_path).unwrap();
    assert_eq!(removed, 2);
    assert!(!stale1.exists());
    assert!(!stale2.exists());
    assert!(safe_file.exists());
    assert!(file_path.exists());
}
