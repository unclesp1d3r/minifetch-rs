# minifetch-rs

[![GitHub License][license-badge]][license-link] [![GitHub Sponsors][sponsors-badge]][sponsors-link]

[![GitHub Actions Workflow Status][ci-badge]][ci-link] [![Deps.rs Repository Dependencies][deps-badge]][deps-link]

[![Codecov][codecov-badge]][codecov-link] [![GitHub issues][issues-badge]][issues-link] [![GitHub last commit][last-commit-badge]][commits-link]

[![Crates.io][crates-badge]][crates-link] [![GitHub Release Date][release-date-badge]][releases-link] [![Crates.io Downloads (latest version)][downloads-badge]][crates-link] [![Crates.io MSRV][msrv-badge]][crates-link]

---

A small neofetch-style system information CLI written in Rust. Single binary, zero configuration, instant output.

## Overview

minifetch-rs displays a clean, boxed summary of your system at a glance: hostname, OS, kernel, uptime, logged-in users, load average, RAM, swap, disk usage, network interfaces, and thermal sensors. Output is colored and aligned in the terminal.

Built for operators who want fast, honest system info without bloat.

- **Single binary** — no runtime dependencies, no config files required
- **Fast** — only refreshes the data it actually displays; no full process table scan
- **Safe** — sanitizes all OS-sourced strings to block terminal escape injection
- **Zero unsafe code** — `unsafe_code = "forbid"` enforced project-wide

## Features

- ASCII banner for the hostname (falls back to bold text for short names)
- Boxed info panel with aligned columns:
  - `User@Host` — centered header row
  - `OS` and `Kernel` version
  - `Uptime` in human-readable form
  - `Users` — currently logged-in accounts
  - `Load` — 1/5/15-minute load averages
  - `RAM` and `Swap` — usage percentage with a visual fill bar
  - `Disk (mountpoint)` — per-device usage with a fill bar (deduplicates bind mounts)
  - `Net (interface)` — MAC address, total Rx/Tx (real interfaces only, no loopback or tunnels)
  - `Temp (sensor)` — up to five thermal sensors in °C
- Timestamp line below the box
- Graceful handling of `SIGPIPE` — `minifetch-rs | head -1` exits cleanly

## Installation

### Pre-built Binaries

Download binaries for Linux (x86\_64, ARM64), macOS (Intel, Apple Silicon), and Windows from [Releases][releases-link].

### From crates.io

```bash
cargo install minifetch-rs
```

### Build from Source

```bash
git clone https://github.com/unclesp1d3r/minifetch-rs.git
cd minifetch-rs
cargo build --release
# Binary at target/release/minifetch-rs
```

## Usage

```bash
minifetch-rs
```

No flags are required. `--help` and `--version` are available:

```bash
minifetch-rs --help
minifetch-rs --version
```

### Example Output

```text
╔═══════════════════════════════════════════════╗
  _ __ ___  _   _ ___  ___ _ ____   _____ _ __
 | '_ ` _ \| | | / __|/ _ \ '__\ \ / / _ \ '__|
 | | | | | | |_| \__ \  __/ |   \ V /  __/ |
 |_| |_| |_|\__, |___/\___|_|    \_/ \___|_|
             |___/
┌──────────────────────────────────────────────┐
│            operator@myserver.local            │
├──────────────────────────────────────────────┤
│ OS     : macOS 15.4.1                         │
│ Kernel : 24.4.0                               │
│ Uptime : 3days 7h 22m 10s                     │
│ Users  : operator                             │
│ Load   : 1.42 1.08 0.97                       │
│ RAM    : 63.24% ████████████░░░░░░░░          │
│ Swap   : 12.50% ██░░░░░░░░░░░░░░░░░░          │
│ Disk (/): 48.10% █████████░░░░░░░░░░░░        │
│ Net (en0): 00:1a:2b:3c:4d:5e (Rx: 1.2 GB, Tx: 340 MB) │
│ Temp (CPU): 51.3°C                            │
└──────────────────────────────────────────────┘
Date: 2026-04-24 09:15:42
```

## Requirements

- Rust 1.89 or later (MSRV)
- No external system libraries required

## Development

This project uses [mise](https://mise.jdx.dev/) for toolchain management and [just](https://github.com/casey/just) for task automation.

```bash
# Install tools
just setup

# Run
cargo run

# Tests
cargo test

# Lint
cargo clippy --all-targets -- -D warnings

# Format
just fmt

# Full check (format + clippy + tests)
just ci
```

## License

Apache-2.0. See [LICENSE](LICENSE).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Read [AI_POLICY.md](AI_POLICY.md) before submitting AI-assisted work.

---

<!-- Badges -->
[license-badge]: https://img.shields.io/github/license/unclesp1d3r/minifetch-rs
[license-link]: https://github.com/unclesp1d3r/minifetch-rs/blob/main/LICENSE
[sponsors-badge]: https://img.shields.io/github/sponsors/unclesp1d3r
[sponsors-link]: https://github.com/sponsors/unclesp1d3r
[ci-badge]: https://img.shields.io/github/actions/workflow/status/unclesp1d3r/minifetch-rs/ci.yml
[ci-link]: https://github.com/unclesp1d3r/minifetch-rs/actions
[deps-badge]: https://deps.rs/repo/github/unclesp1d3r/minifetch-rs/status.svg
[deps-link]: https://deps.rs/repo/github/unclesp1d3r/minifetch-rs
[codecov-badge]: https://codecov.io/gh/unclesp1d3r/minifetch-rs/branch/main/graph/badge.svg
[codecov-link]: https://codecov.io/gh/unclesp1d3r/minifetch-rs
[issues-badge]: https://img.shields.io/github/issues/unclesp1d3r/minifetch-rs
[issues-link]: https://github.com/unclesp1d3r/minifetch-rs/issues
[last-commit-badge]: https://img.shields.io/github/last-commit/unclesp1d3r/minifetch-rs
[commits-link]: https://github.com/unclesp1d3r/minifetch-rs/commits/main
[crates-badge]: https://img.shields.io/crates/v/minifetch-rs
[crates-link]: https://crates.io/crates/minifetch-rs
[release-date-badge]: https://img.shields.io/github/release-date/unclesp1d3r/minifetch-rs
[releases-link]: https://github.com/unclesp1d3r/minifetch-rs/releases
[downloads-badge]: https://img.shields.io/crates/dv/minifetch-rs
[msrv-badge]: https://img.shields.io/crates/msrv/minifetch-rs
