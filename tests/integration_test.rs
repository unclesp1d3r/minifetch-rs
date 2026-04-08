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

/// Sanity check that the rendered output actually contains the rows and
/// box characters we expect. This guards against an empty-output
/// regression where the binary exits 0 but prints nothing useful.
#[test]
fn output_contains_expected_sections() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::cargo_bin("minifetch-rs")?.output()?;
    assert!(output.status.success(), "exit status: {:?}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("OS:"), "missing OS row in output:\n{stdout}");
    assert!(
        stdout.contains("RAM:"),
        "missing RAM row in output:\n{stdout}"
    );
    assert!(
        stdout.contains("Date:"),
        "missing Date footer in output:\n{stdout}"
    );

    // Box-drawing characters may render differently under Windows's
    // default console codepage, so only assert them on non-Windows.
    #[cfg(not(windows))]
    {
        assert!(
            stdout.contains("┌"),
            "missing top-left box char in output:\n{stdout}"
        );
        assert!(
            stdout.contains("└"),
            "missing bottom-left box char in output:\n{stdout}"
        );
    }

    Ok(())
}
