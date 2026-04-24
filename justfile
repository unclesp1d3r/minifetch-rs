# minifetch-rs — developer tasks
# Cross-platform justfile. Windows uses PowerShell, Unix uses bash.
# All cargo/tool invocations go through `mise exec --` so the pinned
# toolchain from mise.toml is used. `just` itself is never wrapped in
# `mise exec --` because that is redundant.

set shell := ["bash", "-cu"]
set windows-shell := ["powershell", "-NoProfile", "-Command"]
set dotenv-load
set ignore-comments

mise_exec := "mise exec --"

# =============================================================================
# GENERAL
# =============================================================================

# Default recipe — list available recipes
default:
    @just --list

# =============================================================================
# SETUP
# =============================================================================

# Development setup — mise handles all tool installation via mise.toml
setup:
    mise trust
    mise install

# =============================================================================
# FORMAT
# =============================================================================

alias format-rust := fmt

# Main format recipe — calls all formatters
format: fmt fmt-justfile

# Format Rust code
fmt:
    @{{ mise_exec }} cargo fmt --all

# Check Rust formatting (CI / pre-commit)
fmt-check:
    @{{ mise_exec }} cargo fmt --all --check

# Format the justfile itself
fmt-justfile:
    @just --fmt --unstable

# Lint justfile formatting
lint-justfile:
    @just --fmt --check --unstable

# =============================================================================
# LINT
# =============================================================================

alias lint-rust := clippy

# Main lint recipe — calls sub-linters
lint: clippy lint-justfile lint-actions

# Run clippy with strict zero-warning policy
clippy:
    @{{ mise_exec }} cargo clippy --all-targets --all-features -- -D warnings

# Validate GitHub Actions workflows (covers both .yml and .yaml)
lint-actions:
    @{{ mise_exec }} actionlint .github/workflows/*.y*ml

# Run clippy with automatic fixes
fix:
    @{{ mise_exec }} cargo clippy --fix --allow-dirty --allow-staged

# =============================================================================
# BUILD
# =============================================================================

# Debug build
build:
    @{{ mise_exec }} cargo build

# Release build
build-release:
    @{{ mise_exec }} cargo build --release

# =============================================================================
# TEST
# =============================================================================

# Run tests with nextest (preferred)
test:
    @{{ mise_exec }} cargo nextest run --no-capture

# Run tests the way CI runs them
test-ci:
    @{{ mise_exec }} cargo nextest run --all-features

# Intentionally NOT part of `check` / `dev` / `ci-check` aggregates:
# minifetch-rs has zero `///` doctest blocks today, so running
# `cargo test --doc` from an aggregate would just compile docs as a
# no-op. Re-add it once the crate starts carrying runnable doc
# examples.

# Run documentation tests (nextest does not support doctests)
test-doc:
    @{{ mise_exec }} cargo test --doc

# =============================================================================
# COVERAGE
# =============================================================================

# Generate lcov coverage report
coverage:
    @{{ mise_exec }} cargo llvm-cov --all-features --lcov --output-path lcov.info

# Check coverage against a floor (adjust threshold as coverage grows)
coverage-check:
    @{{ mise_exec }} cargo llvm-cov --all-features --lcov --output-path lcov.info --fail-under-lines 70

# =============================================================================
# SECURITY
# =============================================================================

# Dependency vulnerability audit
audit:
    @{{ mise_exec }} cargo audit

# License and supply-chain policy check
deny:
    @{{ mise_exec }} cargo deny check

# =============================================================================
# RUN / INSTALL
# =============================================================================

# Run the CLI with optional args
run *args:
    @{{ mise_exec }} cargo run -- {{ args }}

# Install from source into the current cargo bin path
install:
    @{{ mise_exec }} cargo install --path .

# =============================================================================
# CI / WORKFLOW AGGREGATES
# =============================================================================

# Quick development check (fmt + lint + test)
check: fmt-check lint test-ci

# Development workflow: format, lint, test
dev: fmt lint test

# Mirrors .github/workflows/ci.yml (quality + test + test-cross-platform
# + coverage + msrv jobs) and adds `audit` + `deny`, which the GitHub
# workflow does not run yet. Both tools are installed via mise.toml and
# are cheap enough to run locally before pushing.

# Full local CI parity check (adds audit + deny over GitHub CI)
ci-check: fmt-check clippy test-ci build-release coverage audit deny

# =============================================================================
# MAINTENANCE
# =============================================================================

# Update dependencies
update:
    @{{ mise_exec }} cargo update

# Check for outdated dependencies
outdated:
    @{{ mise_exec }} cargo outdated

# Clean build artifacts
clean:
    @{{ mise_exec }} cargo clean

# Show project info
info:
    @echo "minifetch-rs — small neofetch-style system info CLI"
    @{{ mise_exec }} rustc --version
    @{{ mise_exec }} cargo --version
