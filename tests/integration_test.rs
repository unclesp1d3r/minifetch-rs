use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_run() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("minifetch-rs")?;
    cmd.assert().success();
    Ok(())
}
