# AI Code Assistant Configuration

This file outlines the coding standards, architectural patterns, and project layout preferences for projects developed by this user. It serves as a comprehensive guide for the AI Code Assistant to ensure consistency, maintainability, and adherence to established best practices.

## 1. Core Philosophy

- **Framework-First Principle**: Always prefer built-in functionality from frameworks like FastAPI and Pydantic over custom or "clever" solutions. Trust the framework's serialization, validation, and dependency injection mechanisms.
- **Operator-Centric Design**: Projects are built for operators, by operators. This means prioritizing workflows that are efficient, auditable, and functional in contested or airgapped environments.
- **Structured and Versioned Data**: All data models and interactions should be structured, versioned, and non-destructive. Updates should create new versions rather than overwriting existing data.

## 2. Project Structure and Layout

A consistent project structure is maintained across all projects, with clear separation of concerns.

### Standard Rust Application Layout

```text
/
├── src/
│   ├── main.rs          # Entry point, CLI parsing
│   ├── config.rs        # Configuration loading and management
│   ├── api.rs           # API client, if applicable
│   ├── module/
│   │   ├── mod.rs
│   │   └── ...
│   └── utils.rs         # Utility functions
├── tests/               # Integration tests
├── benches/             # Benchmarks
├── examples/            # Usage examples
├── data/                # Static data
├── Cargo.toml           # Project dependencies and metadata
└── README.md            # Project documentation
```

## 3. Technology Stack

The preferred technology stack is consistent across projects:

| Layer        | Technology                                                | Notes                                             |
| ------------ | --------------------------------------------------------- | ------------------------------------------------- |
| **Backend**  | Actix Web, Axum, or Tonic                                 | Async-first design.                               |
| **Database** | PostgreSQL with `sqlx` or `diesel`                        | Async drivers (`tokio-postgres`).                 |
| **CLI**      | `clap` + `ratatui` or `dialoguer`                         | For clean, user-friendly command-line interfaces. |
| **Testing**  | `pytest` (for Python), `cargo test`, `criterion`          | Unit, integration, and benchmark testing.         |
| **CI/CD**    | GitHub Actions                                            | For automated testing, linting, and releases.     |
| **Tooling**  | `cargo` for dependency management, `just` for task running. | `clippy` for linting and `rustfmt` for formatting.|

## 4. Coding Standards and Conventions

### Rust

- **Formatting**: All Rust code must be formatted using `rustfmt`.
- **Linting**: We use `clippy` for static analysis.
- **Naming Conventions**:
  - **Crates, Modules, Files**: `snake_case`
  - **Structs, Enums, Traits**: `PascalCase`
  - **Functions/Methods, Variables**: `snake_case`
- **Error Handling**: Errors must be handled using `Result<T, E>`. The `anyhow` and `thiserror` crates will be used for flexible and structured error handling. `panic!` should not be used for recoverable errors.
- **Concurrency**: Use `tokio` for asynchronous operations. Protect shared state with mutexes from `tokio::sync` where necessary.
- **Logging**: Use `tracing` for structured logging.
- **Testing**: Write unit tests for core logic and place them in the same file as the code being tested, inside a `#[cfg(test)]` module. Integration tests go in the `tests/` directory.

### Commit Messages

- **Conventional Commits**: All commit messages must adhere to the [Conventional Commits](https://www.conventionalcommits.org) specification.
  - **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`.
  - **Scopes**: `(auth)`, `(api)`, `(cli)`, `(models)`, etc.
  - **Breaking Changes**: Indicated with `!` in the header or `BREAKING CHANGE:` in the footer.

## 5. Architectural Patterns

- **Service Layer**: Business logic is encapsulated in service functions/classes, keeping API route handlers thin.
- **Dependency Injection**: Used where appropriate to provide dependencies like database connections and configuration.
- **Schema-Driven Development**: `serde` is used for serialization and deserialization, ensuring type safety.

## 6. Testing Strategy

A three-tier testing architecture is employed to ensure quality:

1. **Unit Tests**: `cargo test` for testing individual components and functions.
2. **Integration Tests**: `cargo test --test '*' -- --test-threads=1` for testing the integration between different components.
3. **End-to-End Tests**: `playwright` or similar tools for testing the application as a whole.

## 7. Agent Behavior and Prompting

- **Clarity and Precision**: The assistant should be direct, professional, and context-aware.
- **Adherence to Rules**: Strictly follow the defined rules for architecture, code style, and testing.
- **Code Generation**: Generated code must conform to all established patterns, including type safety, error handling, and documentation.
- **Tool Usage**: When modifying dependencies, use the appropriate `cargo` commands. When running tasks, use `just`.
- **Project Context**: Never confuse different projects. Maintain a clear understanding of the distinct goals and technology stacks of each.

# Agent Rules <!-- tessl-managed -->

@.tessl/RULES.md follow the [instructions](.tessl/RULES.md)
