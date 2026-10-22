#![cfg(target_os = "linux")]

use std::path::Path;
use std::process::Command;

#[test]
fn notification_backend_reports_unavailable_private_bus() {
    if std::env::var("AVEN_NOTIFICATION_BACKEND_PROBE").as_deref() == Ok("1") {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap();
        let path = address.strip_prefix("unix:path=").unwrap();
        assert!(path.ends_with("/unavailable-bus.sock"));
        assert!(!Path::new(path).exists());
        let result = notify_rust::Notification::new()
            .summary("Aven backend fixture")
            .body("Synthetic missing-bus probe")
            .show();
        assert!(
            result.is_err(),
            "an unavailable private bus must return an error"
        );
        return;
    }

    // Use a new process to avoid mutating global environment or a cached live
    // connection. The private missing socket cannot reach the user's session.
    let directory = tempfile::tempdir().unwrap();
    let address = format!(
        "unix:path={}",
        directory.path().join("unavailable-bus.sock").display()
    );
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "notification_backend_reports_unavailable_private_bus",
            "--nocapture",
        ])
        .env("AVEN_NOTIFICATION_BACKEND_PROBE", "1")
        .env("DBUS_SESSION_BUS_ADDRESS", address)
        .env_remove("DBUS_SESSION_BUS_PID")
        .env_remove("DBUS_STARTER_ADDRESS")
        .env_remove("DBUS_STARTER_BUS_TYPE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "isolated backend probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
