use assert_cmd::prelude::*;
use std::process::Command;

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
///
/// The prior implementation of this test used `Stdio::piped()` + dropping
/// `child.stdout` after spawn, which was racy: on macOS the pipe buffer
/// is 16 KB and minifetch's whole output fits comfortably, so a fast
/// child could finish writing to the buffered pipe *before* the parent
/// dropped the reader — `BrokenPipe` never fired and the test passed for
/// the wrong reason. `os_pipe::pipe()` lets us pre-create a pipe,
/// immediately drop the read end in the parent, and hand the write end
/// to the child. The child's very first stdout write then returns
/// `BrokenPipe` deterministically.
#[test]
fn broken_pipe_exits_zero() -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = os_pipe::pipe()?;
    drop(reader);
    let bin = assert_cmd::cargo::cargo_bin("minifetch-rs");
    let status = Command::new(bin).stdout(writer).status()?;
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

    // Labels inside the box are padded to the widest label with trailing
    // spaces before the `:` (see `BoxLayout::max_label_width`), so assert
    // on ` OS ` / ` RAM ` with bracketing spaces rather than `OS:` /
    // `RAM:`. The " " boundaries also prevent accidentally matching
    // substrings inside other words ("macOS", "RAM"-in-value, etc.).
    assert!(
        stdout.contains(" OS "),
        "missing OS row in output:\n{stdout}"
    );
    assert!(
        stdout.contains(" RAM "),
        "missing RAM row in output:\n{stdout}"
    );
    // The Date row is the unpadded footer outside the box, so it still
    // has the classic `Date: ` format with no padding.
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
