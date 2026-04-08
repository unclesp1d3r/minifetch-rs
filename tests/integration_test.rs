use assert_cmd::prelude::*;
use std::process::{Command, Stdio};

#[test]
fn test_run() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("minifetch-rs")?;
    cmd.assert().success();
    Ok(())
}

/// Regression test for the SIGPIPE → panic bug. Running
/// `minifetch-rs | head -1` (or any pipeline that closes the read end
/// before the binary finishes writing) used to panic with a backtrace.
/// It should now exit cleanly with status 0.
#[test]
fn broken_pipe_exits_zero() -> Result<(), Box<dyn std::error::Error>> {
    let bin = assert_cmd::cargo::cargo_bin("minifetch-rs");
    let mut child = Command::new(bin).stdout(Stdio::piped()).spawn()?;
    // Drop the read end immediately so the next write returns BrokenPipe.
    drop(child.stdout.take());
    let status = child.wait()?;
    assert!(
        status.success(),
        "expected clean exit on closed pipe, got {status:?}"
    );
    Ok(())
}
