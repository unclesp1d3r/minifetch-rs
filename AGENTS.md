# minifetch-rs — AI Code Assistant Configuration

A small neofetch-style system info CLI written in Rust. Single binary, currently `src/main.rs`. This file documents project conventions for AI code assistants.

## Project Snapshot

- **Type**: CLI tool (system info display)
- **Crates**: `clap` (derive), `sysinfo`, `colored`, `console`, `figlet-rs`, `chrono`, `humantime`, `users`, `indicatif`, `anyhow`
- **Dev**: `assert_cmd` for integration tests
- **Not used**: tokio/async, web frameworks, databases, `tracing`, `thiserror`, `serde`, `just`

## 1. Core Philosophy

- **Standard-library first**: Prefer std and well-known crates over clever abstractions. This is a small CLI — keep it simple.
- **Operator-Centric Design**: Built for operators, by operators. Prioritize workflows that are efficient, auditable, and functional in contested or airgapped environments.
- **No unnecessary dependencies**: Each new crate must justify its weight. Don't pull in async runtimes, serialization frameworks, or service layers this project doesn't need.

## 2. Project Structure and Layout

A consistent project structure is maintained across all projects, with clear separation of concerns.

### Current Layout

```text
/
├── src/
│   └── main.rs                  # Entry point, CLI parsing, all logic (split as it grows)
├── tests/
│   └── integration_test.rs      # assert_cmd-based CLI tests
├── Cargo.toml
├── AI_POLICY.md                 # Required reading — see §7
└── CLAUDE.md / AGENTS.md        # Agent instructions
```

When `main.rs` grows beyond ~400 lines, split by feature into modules (`info.rs`, `render.rs`, etc.) — not by type.

## 3. Technology Stack

| Layer       | Technology                                     | Notes                            |
| ----------- | ---------------------------------------------- | -------------------------------- |
| **CLI**     | `clap` (derive)                                | Argument parsing                 |
| **System**  | `sysinfo`, `users`                             | OS / hardware / user info        |
| **Output**  | `colored`, `console`, `figlet-rs`, `indicatif` | Terminal formatting and progress |
| **Time**    | `chrono`, `humantime`                          | Timestamps and durations         |
| **Errors**  | `anyhow`                                       | Application-style error handling |
| **Testing** | `cargo test` + `assert_cmd`                    | Unit + CLI integration           |
| **CI/CD**   | GitHub Actions                                 | Lint, test, release              |
| **Tooling** | `cargo`, `clippy`, `rustfmt`                   |                                  |

## 4. Coding Standards and Conventions

### Rust

- **Formatting**: All Rust code must be formatted using `rustfmt`.
- **Linting**: We use `clippy` for static analysis.
- **Naming Conventions**:
  - **Crates, Modules, Files**: `snake_case`
  - **Structs, Enums, Traits**: `PascalCase`
  - **Functions/Methods, Variables**: `snake_case`
- **Error Handling**: Use `Result<T, E>` with `anyhow::Result` for application errors. No `thiserror` — this project doesn't expose a library API. Never `panic!` on recoverable errors; reserve it for true invariants.
- **Concurrency**: Synchronous. Do not introduce `tokio` or async without strong justification.
- **Output**: Use `println!`/`eprintln!` and the `colored`/`console` crates. No `tracing` — this is a one-shot CLI, not a service.
- **Testing**: Unit tests live in `#[cfg(test)] mod tests` blocks alongside the code. CLI behavior is covered by `assert_cmd` integration tests in `tests/`.

### Commit Messages

- **Conventional Commits**: Adhere to the [Conventional Commits](https://www.conventionalcommits.org) specification.
  - **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`.
  - **Scopes** (project-relevant): `(cli)`, `(info)`, `(render)`, `(deps)`.
  - **Breaking Changes**: Indicated with `!` in the header or `BREAKING CHANGE:` in the footer.
- **DCO sign-off**: Use `git commit -s` (legal attestation — only the contributor adds this).

## 5. Architectural Patterns

This is a small CLI — keep architecture flat. Avoid premature abstraction:

- **No service layer / DI / repositories.** Functions over indirection.
- **Group by feature** when splitting modules (e.g. `os_info`, `cpu_info`, `render`), not by type.
- **Pure functions** for info gathering where possible — easier to test.

## 6. Testing Strategy

1. **Unit tests**: `cargo test` — `#[cfg(test)] mod tests` blocks alongside code.
2. **Integration tests**: `cargo test --test integration_test` — drives the binary via `assert_cmd`.
3. **Lint/format gate**: `cargo fmt --check && cargo clippy -- -D warnings` before commit.

## 7. Common Commands

- `cargo run` — run the CLI
- `cargo build --release` — release build
- `cargo test` — all tests
- `cargo fmt` / `cargo clippy --all-targets -- -D warnings`

## 8. AI Assistance Policy

**Read `AI_POLICY.md` before contributing.** Summary:

- AI assistance is allowed. AI-generated *media* (images, logos, audio, video) is **not**.
- **You own every line you submit** and must be able to explain it without asking the AI to explain it back.
- Disclose AI tool usage in PR descriptions (no fixed format).
- Unreviewed AI output (hallucinated APIs, ignored conventions) gets closed without review.

## 9. Agent Behavior

- **Clarity and Precision**: Direct, professional, context-aware.
- **Stay scoped**: One issue per PR. No "while I was here" cleanup.
- **Match the project**: This is minifetch-rs — a small sync CLI. Don't propose tokio, web frameworks, or DI patterns from other projects.
- **Cargo for deps**: `cargo add` / `cargo remove`, never hand-edit `Cargo.toml` versions.

## Agent Rules <!-- tessl-managed -->

@.tessl/RULES.md follow the [instructions](.tessl/RULES.md)
