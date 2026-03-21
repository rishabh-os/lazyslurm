# Agent Guidelines for LazySlurm

## Project Overview
LazySlurm is a Rust TUI application for monitoring and managing Slurm HPC jobs. It uses ratatui for the terminal UI, tokio for async runtime, and anyhow for error handling.

## Build/Lint/Test Commands

### Development
```bash
cargo run                    # Run the application (use `just dev`)
cargo build                  # Build the project
cargo build --release        # Release build
```

### Testing
```bash
cargo test                   # Run all tests
cargo test <test_name>       # Run a specific test
cargo test -- --nocapture    # Run tests with output
```

### Linting & Formatting
```bash
cargo clippy -- -D warnings   # Lint with Clippy (fail on warnings)
cargo fmt                     # Format code
cargo fmt --check             # Check formatting without changes
```

### Using just (recommended)
```bash
just dev                      # Run in dev mode
just test                     # Run tests
just lint                     # Run clippy linting
just clean                    # Clean build artifacts and Docker
```

### Docker/SLURM Development
```bash
just slurm_up                 # Start SLURM dev container
just slurm_shell              # Shell into SLURM container
just slurm_populate           # Submit test jobs
just slurm_status            # Check SLURM status
just slurm_down               # Stop SLURM container
```

## Code Style Guidelines

### Formatting
- Run `cargo fmt` before committing
- 4-space indentation
- Maximum line length: 100 characters (enforced by default rustfmt)

### Imports
- Group imports by crate: std, external (crates), local (crate::)
- Use `use` statements for frequently used items
- Prefer absolute paths from crate root: `crate::models::Job`
- Example:
  ```rust
  use anyhow::{Context, Result};
  use std::process::Command;
  use tokio::process::Command as TokioCommand;

  use crate::models::{Job, JobState};
  use crate::slurm::SlurmCommands;
  ```

### Naming Conventions
- **Types/Enums**: PascalCase (`JobState`, `SlurmCommands`)
- **Functions/Methods**: snake_case (`parse_squeue_output`, `refresh_jobs`)
- **Variables**: snake_case (`job_list`, `current_user`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Modules**: snake_case (`models/`, `slurm/`)

### Structs and Enums
- Use `#[derive(Debug, Clone, ...)]` for common traits
- Implement `Display` for user-facing string representations
- Implement `From` for conversions from related types
- Use enums for state machines and discrete options
- Example:
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub enum JobState {
      Pending,
      Running,
      Completed,
      Unknown(String),
  }

  impl fmt::Display for JobState {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          match self {
              JobState::Pending => write!(f, "PD"),
              // ...
          }
      }
  }
  ```

### Error Handling
- Use `anyhow::Result<T>` for fallible operations
- Use `anyhow::Context` for enriching errors with context
- Use `anyhow::bail!` for early returns with errors
- Never use `unwrap()` or `expect()` in production code
- Example:
  ```rust
  pub async fn squeue(user: Option<&str>) -> Result<String> {
      let output = cmd.output().await.context("Failed to execute squeue")?;
      if !output.status.success() {
          anyhow::bail!("squeue failed: {}", stderr);
      }
      Ok(output)
  }
  ```

### Async Code
- Use `#[tokio::main]` for the main entry point
- Use `async fn` for functions that perform I/O
- Use tokio channels (`mpsc`, `oneshot`) for async communication
- Prefer `tokio::process::Command` for spawning external processes

### TUI/Ratatui
- Create render functions that take `&mut Frame` and component-specific data
- Use layout helpers to calculate widget positions
- Handle terminal cleanup in drop/panic scenarios
- Example:
  ```rust
  pub fn render_app(frame: &mut Frame, app: &App) {
      let chunks = Layout::default()
          .direction(Direction::Vertical)
          .constraints([...])
          .split(frame.area());
      // ...
  }
  ```

### Module Organization
- `src/lib.rs`: Module exports
- `src/main.rs`: Binary entry point, CLI parsing, terminal setup
- `src/models/`: Data structures (`Job`, `JobList`)
- `src/slurm/`: SLURM command execution and parsing
- `src/ui/`: TUI components, rendering, event handling
- `src/utils/`: Utility functions, future config handling

### Comments
- Do NOT add comments unless explaining non-obvious behavior
- Code should be self-documenting through clear naming
- Document public API behavior in doc comments when needed

### Testing
- Add unit tests in the same file using `#[cfg(test)]` module
- Test parsing logic thoroughly with various input formats
- Mock SLURM commands in integration tests when possible

## CI Requirements
All PRs must pass:
1. `cargo test`
2. `cargo clippy -- -D warnings`
3. `cargo fmt --check`

## Development Tips
- Use `cargo watch` (if installed) for auto-rebuilding during development
- Test with actual SLURM using `just slurm_up` for full integration testing
- The edition is 2024 (latest), enabling recent Rust features
